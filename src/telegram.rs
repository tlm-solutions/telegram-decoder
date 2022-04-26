use std::convert::TryFrom;

enum PackageType {
    StandardResponse = 0,
    AbsoluteLocation = 1,
    MessageAcknowledgements = 2,
    Load = 3,
    RouteRun = 4,
    DriversNo = 5,
    NumberOfTrailer = 6,
    DestinationNumber = 7,
    UnitSerialNumber = 8,
    ApplicationSpecific = 9,
    Spare = 10,
    ReducedMessage = 10,
    StandardMessage = 11,
    StandardMessagePriority = 12,
    StandardMessageLineNumber = 13,
    StandardMessageLineNumberRunNumber = 14,
    NotUsed = 15,
    MaximumMessage = 16
}


impl TryFrom<u8> for PackageType {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == PackageType::StandardMessage as u8 => Ok(PackageType::StandardMessage),
            x if x == PackageType::AbsoluteLocation as u8 => Ok(PackageType::AbsoluteLocation),
            x if x == PackageType::MessageAcknowledgements as u8 => Ok(PackageType::MessageAcknowledgements),

            x if x == PackageType::Load as u8 => Ok(PackageType::Load),
            x if x == PackageType::RouteRun as u8 => Ok(PackageType::RouteRun),
            x if x == PackageType::DriversNo as u8 => Ok(PackageType::DriversNo),
            x if x == PackageType::NumberOfTrailer as u8 => Ok(PackageType::NumberOfTrailer),
            x if x == PackageType::DestinationNumber as u8 => Ok(PackageType::DestinationNumber),
            x if x == PackageType::UnitSerialNumber as u8 => Ok(PackageType::UnitSerialNumber),
            x if x == PackageType::ApplicationSpecific as u8 => Ok(PackageType::ApplicationSpecific),
            x if x == PackageType::Spare as u8 => Ok(PackageType::Spare),
            x if x == PackageType::ReducedMessage as u8 => Ok(PackageType::ReducedMessage),
            x if x == PackageType::StandardMessage as u8 => Ok(PackageType::StandardMessage),
            x if x == PackageType::StandardMessagePriority as u8 => Ok(PackageType::StandardMessagePriority),
            x if x == PackageType::StandardMessageLineNumber as u8 => Ok(PackageType::StandardMessageLineNumber),
            x if x == PackageType::StandardMessageLineNumberRunNumber as u8 => Ok(PackageType::StandardMessageLineNumberRunNumber),
            x if x == PackageType::NotUsed as u8 => Ok(PackageType::NotUsed),
            x if x == PackageType::MaximumMessage as u8 => Ok(PackageType::MaximumMessage),
            x if x == MyEnum::C as i32 => Ok(MyEnum::C),
            _ => Err(()),
        }
    }
}

