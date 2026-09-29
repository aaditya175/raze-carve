-- Wireshark / Tcpdump PCAP Structure Validator
--
-- Validates classic libpcap 24-byte header:
-- Magic (0xD4C3B2A1 or 0xA1B2C3D4) and version 2.4

signature {
    name     = "Wireshark PCAP Capture",
    category = "Forensics",
    header   = "D4C3B2A1",
    max_size = "10GB",

    validate = function(data)
        if #data < 24 then
            return false
        end

        -- Verify major version = 2, minor version = 4
        local ver_major = data:byte(5) + data:byte(6) * 256
        local ver_minor = data:byte(7) + data:byte(8) * 256

        -- In little-endian, major should be 2 and minor 4
        return (ver_major == 2 and ver_minor == 4) or (data:byte(6) == 2 and data:byte(8) == 4)
    end
}
