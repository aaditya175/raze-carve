#!/usr/bin/env python3
import argparse, json, sys, os, hashlib
from pathlib import Path
from datetime import datetime, timezone

try:
    import magic
    MAGIC_AVAILABLE = True
except ImportError:
    MAGIC_AVAILABLE = False

try:
    import exifread
    EXIF_AVAILABLE = True
except ImportError:
    EXIF_AVAILABLE = False

try:
    from PIL import Image
    PIL_AVAILABLE = True
except ImportError:
    PIL_AVAILABLE = False
def parse_args():
    parser = argparse.ArgumentParser(description="SecureForge File Classifier")
    parser.add_argument('--scan-dir', required=True, help="Directory to scan")
    parser.add_argument('--min-size', type=int, default=0, help="Minimum file size in bytes")
    parser.add_argument('--output-json', help="Output JSON Lines file")
    return parser.parse_args()
def detect_mime(path):
    if MAGIC_AVAILABLE:
        try:
            return magic.from_file(str(path), mime=True)
        except Exception:
            pass

    # Inspect file magic bytes fallback when python-magic is unavailable
    try:
        with open(path, 'rb') as f:
            header = f.read(32)
            if len(header) >= 4:
                if header.startswith(b'%PDF-'):
                    return 'application/pdf'
                if header.startswith(b'\xff\xd8\xff'):
                    return 'image/jpeg'
                if header.startswith(b'\x89PNG\r\n\x1a\n'):
                    return 'image/png'
                if header.startswith(b'GIF87a') or header.startswith(b'GIF89a'):
                    return 'image/gif'
                if header.startswith(b'PK\x03\x04') or header.startswith(b'PK\x05\x06'):
                    return 'application/zip'
                if header.startswith(b'SQLite format 3\x00'):
                    return 'application/x-sqlite3'
                if header.startswith(b'MZ'):
                    return 'application/x-msdownload'
                if header.startswith(b'\x7fELF'):
                    return 'application/x-executable'
                if header.startswith(b'\x1f\x8b'):
                    return 'application/gzip'
                if header.startswith(b'7z\xbc\xaf\x27\x1c'):
                    return 'application/x-7z-compressed'
                if header.startswith(b'RIFF') and len(header) >= 12:
                    fourcc = header[8:12]
                    if fourcc == b'WEBP':
                        return 'image/webp'
                    if fourcc == b'AVI ':
                        return 'video/x-msvideo'
                    if fourcc == b'WAVE':
                        return 'audio/wav'
    except Exception:
        pass
    
    ext = os.path.splitext(path)[1].lower()
    ext_map = {
        '.txt': 'text/plain', '.csv': 'text/csv', '.md': 'text/markdown',
        '.json': 'application/json', '.xml': 'application/xml',
        '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.png': 'image/png',
        '.gif': 'image/gif', '.webp': 'image/webp', '.bmp': 'image/bmp',
        '.mp4': 'video/mp4', '.mkv': 'video/x-matroska', '.mp3': 'audio/mpeg',
        '.wav': 'audio/wav', '.flac': 'audio/flac',
        '.pdf': 'application/pdf', '.doc': 'application/msword',
        '.docx': 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
        '.xls': 'application/vnd.ms-excel',
        '.xlsx': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
        '.zip': 'application/zip', '.tar': 'application/x-tar', '.gz': 'application/gzip',
        '.7z': 'application/x-7z-compressed',
        '.exe': 'application/x-msdownload', '.dll': 'application/x-msdownload',
        '.db': 'application/x-sqlite3', '.sqlite': 'application/x-sqlite3',
        '.sqlite3': 'application/x-sqlite3', '.sql': 'application/sql',
        '.log': 'text/plain', '.ini': 'text/plain', '.cfg': 'text/plain'
    }
    return ext_map.get(ext, 'application/octet-stream')
def get_category(mime):
    if mime.startswith('image/') or mime.startswith('video/') or mime.startswith('audio/'):
        return 'Media'
    if mime in [
        'application/pdf', 'text/plain', 'text/csv', 'text/markdown',
        'application/msword',
        'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
        'application/vnd.ms-excel',
        'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
        'application/json', 'application/xml'
    ]:
        return 'Documents'
    if mime in [
        'application/zip', 'application/x-tar', 'application/gzip',
        'application/x-7z-compressed', 'application/x-bzip2', 'application/x-xz'
    ]:
        return 'Archives'
    if 'sqlite' in mime or 'sql' in mime:
        return 'Databases'
    if 'executable' in mime or 'x-msdownload' in mime or 'x-elf' in mime:
        return 'Executables'
    return 'Unknown'
def extract_exif(path):
    if not EXIF_AVAILABLE:
        return {}
    try:
        with open(path, 'rb') as f:
            tags = exifread.process_file(f, details=False)
            return {k: str(v) for k, v in tags.items() if k in ['EXIF DateTimeOriginal', 'Image Make', 'Image Model']}
    except Exception:
        return {}
def compute_dhash(path):
    if not PIL_AVAILABLE:
        return None
    try:
        with Image.open(path) as img:
            # Composite images with transparency over white background for consistent perceptual hashing
            if img.mode in ('RGBA', 'LA') or (img.mode == 'P' and 'transparency' in img.info):
                rgba = img.convert('RGBA')
                bg = Image.new('RGBA', rgba.size, (255, 255, 255, 255))
                img = Image.alpha_composite(bg, rgba).convert('L')
            else:
                img = img.convert('L')
            img = img.resize((9, 8), Image.Resampling.LANCZOS)
            diff = []
            for row in range(8):
                for col in range(8):
                    pixel_left = img.getpixel((col, row))
                    pixel_right = img.getpixel((col + 1, row))
                    diff.append(pixel_left > pixel_right)
            
            decimal_value = 0
            hex_string = []
            for index, value in enumerate(diff):
                if value:
                    decimal_value += 2**(index % 8)
                if (index % 8) == 7:
                    hex_string.append(hex(decimal_value)[2:].rjust(2, '0'))
                    decimal_value = 0
            return ''.join(hex_string)
    except Exception:
        return None

def hamming_distance(hash1, hash2):
    if not hash1 or not hash2 or len(hash1) != len(hash2):
        return 64
    try:
        val1 = int(hash1, 16)
        val2 = int(hash2, 16)
        return bin(val1 ^ val2).count('1')
    except Exception:
        return 64
def compute_sha256(path):
    h = hashlib.sha256()
    try:
        with open(path, 'rb') as f:
            while chunk := f.read(8192):
                h.update(chunk)
        return h.hexdigest()
    except Exception:
        return None
def classify_file(path):
    mime = detect_mime(path)
    return {
        'path': str(path),
        'filename': os.path.basename(path),
        'size_bytes': os.path.getsize(path),
        'mime': mime,
        'category': get_category(mime),
        'sha256': compute_sha256(path),
        'dhash': compute_dhash(path) if mime.startswith('image/') else None,
        'exif': extract_exif(path) if mime.startswith('image/') else {},
        'classified_at': datetime.now(timezone.utc).isoformat()
    }
def main():
    args = parse_args()
    count = 0
    out_f = open(args.output_json, 'w') if args.output_json else sys.stdout
    try:
        for root, dirs, files in os.walk(args.scan_dir):
            for file in files:
                filepath = os.path.join(root, file)
                try:
                    # Refuse to follow symlinks during classification. A
                    # malicious symlink inside the scan dir could otherwise
                    # point at an arbitrary file outside the scope (e.g. a
                    # sensitive system file) and get read/exfiltrated into
                    # the classification output.
                    if os.path.islink(filepath):
                        continue
                    size = os.path.getsize(filepath)
                    if size < args.min_size: continue
                    record = classify_file(filepath)
                    out_f.write(json.dumps(record) + '\n')
                    count += 1
                except Exception as e:
                    print(f"Error processing {filepath}: {e}", file=sys.stderr)
    finally:
        if args.output_json and out_f != sys.stdout:
            out_f.close()
    print(f"Classification summary: processed {count} files", file=sys.stderr)

if __name__ == '__main__':
    main()
