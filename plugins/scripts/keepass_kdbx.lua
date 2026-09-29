-- KeePass 2.x Password Database (.kdbx) Structure Validator
--
-- Validates KDBX signature: 0x03D9A29A 0x67FB4BB5
-- and verifies database version field (v3.x or v4.x)

signature {
    name     = "KeePass 2.x Database",
    category = "Crypto",
    header   = "03D9A29A67FB4BB5",
    max_size = "250MB",

    validate = function(data)
        if #data < 12 then
            return false
        end

        -- Check KeePass Signature 1 (0x9AA2D903) & 2 (0xB54BFB67)
        local sig = { 0x03, 0xD9, 0xA2, 0x9A, 0x67, 0xFB, 0x4B, 0xB5 }
        for i = 1, 8 do
            if data:byte(i) ~= sig[i] then
                return false
            end
        end

        -- Format minor & major version at bytes 9..12
        local major_ver = data:byte(11) + data:byte(12) * 256
        -- KeePass 2.x uses file version 2, 3, or 4
        return major_ver >= 2 and major_ver <= 4
    end
}
