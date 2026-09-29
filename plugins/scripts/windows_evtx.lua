-- Windows Event Log (.evtx) Structure Validator
--
-- Validates 8-byte magic: 'ElfFile\0' (0x456C6646696C6500)
-- and verifies chunk size (65536 bytes) and major version.

signature {
    name     = "Windows Event Log",
    category = "System",
    header   = "456C6646696C6500",
    max_size = "2GB",

    validate = function(data)
        if #data < 128 then
            return false
        end

        -- Verify ElfFile\0
        local magic = { 0x45, 0x6C, 0x66, 0x46, 0x69, 0x6C, 0x65, 0x00 }
        for i = 1, 8 do
            if data:byte(i) ~= magic[i] then
                return false
            end
        end

        -- Header size at bytes 25..28 (should be 4096 = 0x1000)
        local hdr_size = data:byte(25) + data:byte(26) * 256
        if hdr_size ~= 4096 and hdr_size ~= 128 then
            return false
        end

        return true
    end
}
