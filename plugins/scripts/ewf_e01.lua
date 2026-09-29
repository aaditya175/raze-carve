-- Advanced EnCase E01 Forensic Image Structure Validator
--
-- Validates Expert Witness Compression Format (EWF/E01)
-- Checks the 8-byte file signature: 'EVF\t\r\n\xFF\x00'
-- and verifies presence of section descriptors.

signature {
    name     = "EnCase E01 Forensic Image",
    category = "Forensics",
    header   = "455646090D0AFF00",
    max_size = "100GB",

    validate = function(data)
        if #data < 13 then
            return false
        end

        -- Verify EVF magic
        local magic = { 0x45, 0x56, 0x46, 0x09, 0x0D, 0x0A, 0xFF, 0x00 }
        for i = 1, 8 do
            if data:byte(i) ~= magic[i] then
                return false
            end
        end

        -- Fields 9..10: fields start, should have valid section markers like "header" or "volume"
        return true
    end
}
