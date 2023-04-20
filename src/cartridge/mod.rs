/// The cartridge header contains the following information:
/// https://gbdev.io/pandocs/The_Cartridge_Header.html#the-cartridge-header
/// Entry Point - $0100—$0103
/// Nintendo Logo - $0104—$0133
/// Title - $0134—$0142
/// Manufacturer Code - $013F—$0142
/// CGB Flag - $0143
/// New Licensee Code - $0144—$0145
/// SGB Flag - $0146
/// Cartridge Type - $0147 (MBC)
/// ROM Size - $0148
/// RAM Size - $0149
/// Destination Code - $014A
/// Old Licensee Code - $014B
/// Mask ROM Version Number - $014C
/// Header Checksum - $014D
/// Global Checksum - $014E—$014F
pub struct CartridgeHeader {
    entry_point: [u8; 4],
    nintendo_logo: [u8; 48],
    title: [u8; 16],
    manufacturer_code: [u8; 4],
    cgb_flag: u8,
    new_licensee_code: [u8; 2],
    sgb_flag: u8,
    cartridge_type: u8,
    rom_size: u8,
    ram_size: u8,
    destination_code: u8,
    old_licensee_code: u8,
    mask_rom_version_number: u8,
    header_checksum: u8,
    global_checksum: [u8; 2],
}

/// New Licensee Code
/// This is only used if the Old Licensee Code is $33
pub enum NewLicenseeCode {
    None = 0x00,
    NintendoRD1 = 0x01,
    Capcom = 0x08,
    ElectronicArts = 0x13,
    HudsonSoft = 0x18,
    BAi = 0x19,
    Kss = 0x20,
    Pow = 0x22,
    PCmComplete = 0x24,
    SanX = 0x25,
    KemcoJapan = 0x28,
    Seta = 0x29,
    Viacom = 0x30,
    Nintendo = 0x31,
    Bandai = 0x32,
    OceanAcclaim = 0x33,
    Konami = 0x34,
    Hector = 0x35,
    Taito = 0x37,
    Hudson = 0x38,
    Banpresto = 0x39,
    Ubisoft = 0x41,
    Atlus = 0x42,
    Malibu = 0x44,
    Angel = 0x46,
    BulletProof = 0x47,
    Irem = 0x49,
    Absolute = 0x50,
    Acclaim = 0x51,
    Activision = 0x52,
    AmericanSammy = 0x53,
    Konami2 = 0x54,
    HiTechEnt = 0x55,
    Ljn = 0x56,
    Matchbox = 0x57,
    Mattel = 0x58,
    MiltonBradley = 0x59,
    Titus = 0x60,
    Virgin = 0x61,
    LucasArts = 0x64,
    Ocean = 0x67,
    ElectronicArts2 = 0x69, // nice.
    Infogrames = 0x70,
    Interplay = 0x71,
    Broderbund = 0x72,
    Sculptured = 0x73,
    Sci = 0x75,
    Thq = 0x78,
    Accolade = 0x79,
    Misawa = 0x80,
    Lozc = 0x83,
    TokumaShotenIntermedia = 0x86,
    TsukudaOriginal = 0x87,
    Chunsoft = 0x91,
    VideoSystem = 0x92,
    OceanAcclaim2 = 0x93,
    Varie = 0x95,
    YonezawaSpal = 0x96,
    Kaneko = 0x97,
    PackInSoft = 0x99,
    KonamiYuGiOh = 0xA4,
}
