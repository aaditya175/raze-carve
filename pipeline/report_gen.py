#!/usr/bin/env python3
import argparse, json, sys, os, hashlib, subprocess
from pathlib import Path
from datetime import datetime

try:
    from weasyprint import HTML
    WEASYPRINT_AVAILABLE = True
except ImportError:
    WEASYPRINT_AVAILABLE = False

try:
    from jinja2 import Environment, FileSystemLoader, select_autoescape
except ImportError:
    pass

def parse_args():
    parser = argparse.ArgumentParser(description="SecureForge Report Generator")
    parser.add_argument('--input', required=True, help="Input JSON file")
    parser.add_argument('--template', required=True, choices=['erasure', 'recovery', 'custody'])
    parser.add_argument('--output', required=True, help="Output PDF or HTML path")
    parser.add_argument('--investigator', help="Investigator name")
    parser.add_argument('--case-id', help="Case ID")
    parser.add_argument('--tsa-token', help="TSA token path")
    return parser.parse_args()

def load_audit_data(path):
    try:
        with open(path, 'r') as f:
            data = json.load(f)
            return data
    except Exception as e:
        print(f"Error loading audit data: {e}", file=sys.stderr)
        sys.exit(1)

# Allowlist of template variables consumed by the bundled report templates.
# Any other (attacker-influenced) key present in the audit JSON is stripped
# before rendering so it cannot pollute the Jinja2 context or override a
# template global / reserved name.
ALLOWED_CONTEXT_KEYS = {
    'case_id', 'date', 'investigator', 'method', 'passes',
    'device_model', 'device_serial', 'device_capacity', 'device_interface',
    'entropy', 'device', 'files', 'recovered_files', 'tsa_token', 'tsa_status',
    'target', 'operation', 'status', 'verified', 'timestamp', 'created_at',
    'passes_completed', 'entries', 'hash_chain', 'summary', 'root_hash',
    'chain_verified', 'chain_tip', 'embedded_css',
}

def sanitize_context(data):
    return {k: v for k, v in data.items() if k in ALLOWED_CONTEXT_KEYS}

def latex_escape(s):
    if s is None:
        return ''
    s = str(s)
    s = s.replace('\\', r'\textbackslash{}')
    for char, rep in [
        ('&', r'\&'), ('%', r'\%'), ('$', r'\$'), ('#', r'\#'),
        ('_', r'\_'), ('{', r'\{'), ('}', r'\}'),
        ('~', r'\textasciitilde{}'), ('^', r'\textasciicircum{}')
    ]:
        s = s.replace(char, rep)
    return s

def chunk_hash(val):
    if not val:
        return ""
    s = str(val).strip()
    if len(s) >= 32 and all(c in "0123456789abcdefABCDEF" for c in s):
        return " ".join(s[i:i+8] for i in range(0, len(s), 8))
    return latex_escape(s)

def render_latex(data, template_name, template_dir):
    try:
        env = Environment(
            loader=FileSystemLoader(template_dir),
            autoescape=False,
            comment_start_string='(%#',
            comment_end_string='#%)',
        )
        env.filters['tex'] = latex_escape
        env.filters['chunk_hash'] = chunk_hash
        env.filters['tex_hash'] = chunk_hash
        tex_file = f"{template_name}_report.tex"
        if template_name == 'erasure':
            tex_file = f"{template_name}_certificate.tex"
        elif template_name in ['custody', 'chain_of_custody']:
            tex_file = "chain_of_custody.tex"

        target_tex = Path(template_dir) / tex_file
        if target_tex.exists():
            template = env.get_template(tex_file)
            ctx = sanitize_context(data)
            return template.render(**ctx)
    except Exception as e:
        print(f"LaTeX template render note: {e}", file=sys.stderr)
    return None

def generate_latex_pdf(tex_str, output_path):
    import shutil, tempfile
    pdflatex = shutil.which('pdflatex') or shutil.which('xelatex')
    if not pdflatex:
        return None

    try:
        with tempfile.TemporaryDirectory() as td:
            tex_file = Path(td) / "report.tex"
            tex_file.write_text(tex_str, encoding='utf-8')
            cmd = [pdflatex, '-interaction=nonstopmode', '-halt-on-error', 'report.tex']
            # Run twice for TOC, table labels, and page cross-references
            subprocess.run(cmd, cwd=td, capture_output=True, text=True)
            res = subprocess.run(cmd, cwd=td, capture_output=True, text=True)
            generated_pdf = Path(td) / "report.pdf"
            if res.returncode == 0 and generated_pdf.exists() and generated_pdf.stat().st_size > 0:
                shutil.copy(generated_pdf, output_path)
                return output_path
            else:
                print(f"LaTeX compilation failed (code {res.returncode}):\n{res.stdout[-1500:]}", file=sys.stderr)
                print("Falling back to HTML renderer", file=sys.stderr)
    except Exception as e:
        print(f"LaTeX pipeline exception: {e}, falling back to HTML renderer", file=sys.stderr)
    return None

def render_html(data, template_name, template_dir):
    try:
        env = Environment(
            loader=FileSystemLoader(template_dir),
            autoescape=select_autoescape(['html', 'xml'])
        )
    except (TypeError, NameError):
        env = Environment(loader=FileSystemLoader(template_dir), autoescape=True)

    template_file = f"{template_name}_report.html"
    if template_name == 'erasure':
        template_file = f"{template_name}_certificate.html"
    elif template_name in ['custody', 'chain_of_custody']:
        template_file = "chain_of_custody.html"
    try:
        template = env.get_template(template_file)
        css_file = Path(template_dir) / "styles.css"
        embedded_css = css_file.read_text(encoding='utf-8') if css_file.exists() else ""
        ctx = sanitize_context(data)
        ctx['embedded_css'] = embedded_css
        return template.render(**ctx)
    except Exception as e:
        print(f"Error rendering template: {e}", file=sys.stderr)
        sys.exit(1)

def generate_report(html_str, output_path):
    suffix = Path(output_path).suffix.lower()
    if suffix == '.html':
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(html_str)
        return output_path
    if suffix != '.pdf':
        raise ValueError('Output path must end in .pdf or .html')

    # 1. Primary engine: Headless Chromium / Chrome for Overleaf / LaTeX-grade typography
    import shutil
    for browser in ['chromium', 'google-chrome', 'google-chrome-stable', 'chromium-browser']:
        if shutil.which(browser):
            try:
                import tempfile
                with tempfile.NamedTemporaryFile(mode='w', suffix='.html', delete=False, encoding='utf-8') as tf:
                    tf.write(html_str)
                    tf_path = tf.name
                res = subprocess.run([
                    browser,
                    '--headless',
                    '--disable-gpu',
                    '--no-pdf-header-footer',
                    f'--print-to-pdf={output_path}',
                    tf_path
                ], capture_output=True)
                if os.path.exists(tf_path):
                    os.unlink(tf_path)
                if res.returncode == 0 and os.path.exists(output_path) and os.path.getsize(output_path) > 0:
                    return output_path
            except Exception as e:
                print(f"Browser PDF renderer notice: {e}", file=sys.stderr)

    # 2. Secondary engine: WeasyPrint if available
    if WEASYPRINT_AVAILABLE:
        try:
            HTML(string=html_str).write_pdf(output_path)
            return output_path
        except Exception as e:
            print(f"WeasyPrint error: {e}", file=sys.stderr)

    html_path = str(Path(output_path).with_suffix('.html'))
    print(f"Warning: PDF renderer not available or failed. Saving as HTML to {html_path}", file=sys.stderr)
    with open(html_path, 'w', encoding='utf-8') as f:
        f.write(html_str)
    return html_path

def compute_report_hash(path):
    if not os.path.exists(path):
        return None
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        while chunk := f.read(8192):
            h.update(chunk)
    return h.hexdigest()

def main():
    args = parse_args()
    data = load_audit_data(args.input)
    if args.investigator:
        data['investigator'] = args.investigator
    elif not data.get('investigator'):
        data['investigator'] = 'Forensic Operator'

    if args.case_id:
        data['case_id'] = args.case_id
    elif not data.get('case_id'):
        data['case_id'] = 'N/A'

    # Map aliases and fallbacks
    if 'recovered_files' not in data and 'files' in data:
        data['recovered_files'] = data['files']
    elif 'files' not in data and 'recovered_files' in data:
        data['files'] = data['recovered_files']

    if not data.get('date'):
        data['date'] = data.get('timestamp') or data.get('created_at') or datetime.now().strftime('%Y-%m-%d %H:%M:%S UTC')

    if not data.get('device'):
        data['device'] = data.get('target', 'Unknown Evidence Device')

    if not data.get('passes'):
        data['passes'] = data.get('passes_completed', 1)

    if not data.get('device_model'):
        data['device_model'] = data.get('target', 'Forensic Disk')

    data.setdefault('device_serial', 'N/A')
    data.setdefault('device_capacity', 'N/A')
    data.setdefault('device_interface', 'SATA/NVMe')
    data.setdefault('entropy', 'N/A')

    # Wire RFC 3161 TSA token if provided
    if args.tsa_token and os.path.exists(args.tsa_token):
        try:
            with open(args.tsa_token, 'rb') as f:
                token_bytes = f.read()
            token_hash = hashlib.sha256(token_bytes).hexdigest()
            data['tsa_token'] = f"{args.tsa_token} (SHA-256: {token_hash[:16]}...)"
            data['tsa_status'] = (
                f"RFC 3161 token supplied ({len(token_bytes)} bytes; SHA-256: {token_hash[:16]}...). "
                "NOT VERIFIED: signer trust, token status, and message imprint were not checked."
            )
        except Exception as e:
            data['tsa_status'] = f"TSA Token Error: {e}"
    elif args.tsa_token:
        data['tsa_status'] = f"TSA Token Not Found: {args.tsa_token}"

    template_dir = os.path.join(os.path.dirname(__file__), 'templates')
    suffix = Path(args.output).suffix.lower()

    final_output = None
    # 1. Native LaTeX compiler (primary high-grade academic publication engine)
    if suffix == '.pdf':
        tex_str = render_latex(data, args.template, template_dir)
        if tex_str:
            final_output = generate_latex_pdf(tex_str, args.output)

    # 2. Fallback to HTML-to-PDF / direct HTML
    if not final_output:
        html_str = render_html(data, args.template, template_dir)
        try:
            final_output = generate_report(html_str, args.output)
        except ValueError as e:
            print(f"Error generating report: {e}", file=sys.stderr)
            sys.exit(1)
    
    file_hash = compute_report_hash(final_output)
    
    print(json.dumps({"status": "ok", "output": final_output, "hash": file_hash}))

if __name__ == '__main__':
    main()
