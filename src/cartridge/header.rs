use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
/*
 The cartridge header contains the following information:
 https://gbdev.io/pandocs/The_Cartridge_Header.html#the-cartridge-header
 Entry Point - $0100—$0103
 Nintendo Logo - $0104—$0133
 Title - $0134—$0142
 Manufacturer Code - $013F—$0142
 CGB Flag - $0143
 New Licensee Code - $0144—$0145
 SGB Flag - $0146
 Cartridge Type - $0147 (MBC)
 ROM Size - $0148
 RAM Size - $0149
 Destination Code - $014A
 Old Licensee Code - $014B
 Mask ROM Version Number - $014C
 Header Checksum - $014D
 Global Checksum - $014E—$014F
*/

/// Cartridge Type
/// Indicates what kind of hardware is used in the cartridge, most importantly the Memory Bank Controller (MBC).
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum CartridgeType {
    RomOnly = 0x00,
    Mbc1 = 0x01,
    Mbc1Ram = 0x02,
    Mbc1RamBattery = 0x03,
    Mbc2 = 0x05,
    Mbc2Battery = 0x06,
    RomRam = 0x08,
    RomRamBattery = 0x09,
    Mmm01 = 0x0B,
    Mmm01Ram = 0x0C,
    Mmm01RamBattery = 0x0D,
    Mbc3TimerBattery = 0x0F,
    Mbc3TimerRamBattery = 0x10,
    Mbc3 = 0x11,
    /* NOTE: MBC3 with 64 KiB of SRAM refers to MBC30,
    used only in Pocket Monsters: Crystal Version (the Japanese version of Pokémon Crystal Version).
    */
    Mbc3Ram = 0x12,
    Mbc3RamBattery = 0x13,
    Mbc5 = 0x19,
    Mbc5Ram = 0x1A,
    Mbc5RamBattery = 0x1B,
    Mbc5Rumble = 0x1C,
    Mbc5RumbleRam = 0x1D,
    Mbc5RumbleRamBattery = 0x1E,
    Mbc6 = 0x20,
    Mbc7SensorRumbleRamBattery = 0x22,
    PocketCamera = 0xFC,
    BandaiTama5 = 0xFD,
    HuC3 = 0xFE,
    HuC1RamBattery = 0xFF,
}

/// ROM Size
/// The ROM size is usually defined by the following formula:
/// 32KiB x (1 << value).
/// The number of banks is then calculated by dividing the ROM size by 16KiB.
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RomSize {
    Rom32Kb = 0x00,
    Rom64Kb = 0x01,
    Rom128Kb = 0x02,
    Rom256Kb = 0x03,
    Rom512Kb = 0x04,
    Rom1Mb = 0x05,
    Rom2Mb = 0x06,
    Rom4Mb = 0x07,
    Rom8Mb = 0x08,
    /* NOTE:
    1.1 Mb, 1.2mb, and 1.5 Mb are nly listed in unofficial docs.
    No cartridges or ROM files using these sizes are known. A
    s the other ROM sizes are all powers of 2, these are likely inaccurate.
    The source of these values is unknown.
     */
    Rom1_1Mb = 0x52,
    Rom1_2Mb = 0x53,
    Rom1_5Mb = 0x54,
}

/// RAM Size
/// NOTE: If the cartridge type does not have RAM in its name, the RAM size is 0.
/// This includes the MBC2, which has 512 x 4 bits of RAM (built directly into the mapper).
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RamSize {
    None = 0x00,
    Kb2Unused = 0x01,
    Kb8 = 0x02,
    Kb32 = 0x03,
    Kb128 = 0x04,
    Kb64 = 0x05,
}

/// Destination Code
/// This is used to determine if the game is for the Japanese market or the international market.
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum DestinationCode {
    Japan = 0x00,
    Overseas = 0x01,
}

/// New Licensee Codes
/// This is only used if the Old Licensee Code is 0x33
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
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
    Konami54 = 0x54,
    HiTechEnt = 0x55,
    Ljn = 0x56,
    Matchbox = 0x57,
    Mattel = 0x58,
    MiltonBradley = 0x59,
    Titus = 0x60,
    Virgin = 0x61,
    LucasArts = 0x64,
    Ocean = 0x67,
    ElectronicArts69 = 0x69, // nice.
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
    OceanAcclaim93 = 0x93,
    Varie = 0x95,
    YonezawaSpal = 0x96,
    Kaneko = 0x97,
    PackInSoft = 0x99,
    KonamiYuGiOh = 0xA4,
}

/// Old Licensee Codes
/// Used in older pre-SGB cartridges.
/// Use New Licensee code if value is 0x33.
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum OldLicenseeCode {
    None = 0x00,
    Nintendo = 0x01,
    Capcom = 0x08,
    HotB = 0x09,
    Jaleco = 0x0A,
    CoconutsJapan = 0x0B,
    EliteSystems = 0x0C,
    ElectronicArts = 0x13,
    Hudsonsoft = 0x18,
    ItcEntertainment = 0x19,
    Yanoman = 0x1A,
    JapanClary = 0x1D,
    VirginInteractive = 0x1F,
    PcmComplete = 0x24,
    SanX = 0x25,
    KotobukiSystems = 0x28,
    Seta = 0x29,
    Infogrames = 0x30,
    Nintendo31 = 0x31,
    Bandai = 0x32,
    UseNewLicenseeCode = 0x33,
    Konami = 0x34,
    HectorSoft = 0x35,
    Capcom38 = 0x38,
    Banpresto = 0x39,
    EntertainmentI = 0x3C,
    Gremlin = 0x3E,
    Ubisoft = 0x41,
    Atlus = 0x42,
    Malibu = 0x44,
    Angel = 0x46,
    SpectrumHoloby = 0x47,
    Irem = 0x49,
    VirginInteractive4A = 0x4A,
    Malibu4D = 0x4D,
    UsGold = 0x4F,
    Absolute = 0x50,
    Acclaim = 0x51,
    Activision = 0x52,
    AmericanSammy = 0x53,
    GameTek = 0x54,
    ParkPlace = 0x55,
    Ljn = 0x56,
    Matchbox = 0x57,
    MiltonBradley = 0x59,
    Mindscape = 0x5A,
    Romstar = 0x5B,
    NaxatSoft = 0x5C,
    Tradewest = 0x5D,
    Titus = 0x60,
    VirginInteractive61 = 0x61,
    OceanInteractive = 0x67,
    ElectronicArts69 = 0x69,
    EliteSystems6E = 0x6E,
    ElectroBrain = 0x6F,
    Infogrames70 = 0x70,
    Interplay = 0x71,
    Broderbund = 0x72,
    SculpteredSoft = 0x73,
    TheSalesCurve = 0x75,
    Thq = 0x78,
    Accolade = 0x79,
    TriffixEntertainment = 0x7A,
    Microprose = 0x7C,
    Kemco = 0x7F,
    MisawaEntertainment = 0x80,
    Lozc = 0x83,
    TokumaShotenIntermedia = 0x86,
    BulletProofSoftware = 0x8B,
    VicTokai = 0x8C,
    Ape = 0x8E,
    Imax = 0x8F,
    Chunsoft = 0x91,
    VideoSystem = 0x92,
    TsubarayaProductions = 0x93,
    Varie = 0x95,
    YonezawaSPal = 0x96,
    Kaneko = 0x97,
    Arc = 0x99,
    NihonBussan = 0x9A,
    Tecmo = 0x9B,
    Imagineer = 0x9C,
    Banpresto9D = 0x9D,
    Nova = 0x9F,
    HoriElectric = 0xA1,
    BandaiA2 = 0xA2,
    KonamiA4 = 0xA4,
    Kawada = 0xA6,
    Takara = 0xA7,
    TechnosJapan = 0xA9,
    BroderbundAA = 0xAA,
    ToeiAnimation = 0xAC,
    Toho = 0xAD,
    Namco = 0xAF,
    AcclaimB0 = 0xB0,
    AsciiOrNexsoft = 0xB1,
    BandaiB2 = 0xB2,
    SquareEnix = 0xB4,
    HalLaboratory = 0xB6,
    Snk = 0xB7,
    PonyCanyon = 0xB9,
    CultureBrain = 0xBA,
    Sunsoft = 0xBB,
    SonyImagesoft = 0xBD,
    Sammy = 0xBF,
    Taito = 0xC0,
    KemcoC2 = 0xC2,
    Squaresoft = 0xC3,
    TokumaShotenIntermediaC4 = 0xC4,
    DataEast = 0xC5,
    Tonkinhouse = 0xC6,
    Koei = 0xC8,
    Ufl = 0xC9,
    Ultra = 0xCA,
    Vap = 0xCB,
    UseCorporation = 0xCC,
    Meldac = 0xCD,
    PonyCanyonOr = 0xCE,
    AngelCF = 0xCF,
    TaitoD0 = 0xD0,
    Sofel = 0xD1,
    Quest = 0xD2,
    SigmaEnterprises = 0xD3,
    AskKodansha = 0xD4,
    NaxatSoftD6 = 0xD6,
    CopyaSystem = 0xD7,
    BanprestoD9 = 0xD9,
    Tomy = 0xDA,
    LjnDA = 0xDB,
    Ncs = 0xDD,
    Human = 0xDE,
    Altron = 0xDF,
    JalecoE0 = 0xE0,
    TowaChiki = 0xE1,
    Yutaka = 0xE2,
    VarieE3 = 0xE3,
    Epoch = 0xE5,
    Athena = 0xE7,
    AsmikAceEntertainment = 0xE8,
    Natsume = 0xE9,
    KingRecords = 0xEA,
    AtlusEB = 0xEB,
    EpicSonyRecords = 0xEC,
    Igs = 0xEE,
    AWave = 0xF0,
    ExtremeEntertainment = 0xF3,
    LjnFF = 0xFF,
}
