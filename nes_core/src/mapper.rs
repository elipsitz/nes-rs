use super::cartridge::Cartridge;
use serde::{ser::SerializeTuple, Deserialize, Deserializer, Serialize, Serializer};

use super::{mapper_mmc1::MapperMmc1, mapper_mmc3::MapperMmc3, mapper_nrom::MapperNrom};

erased_serde::serialize_trait_object!(Mapper);

pub trait Mapper: erased_serde::Serialize {
    fn peek(&mut self, addr: u16) -> u8;
    fn poke(&mut self, addr: u16, val: u8);

    fn get_id(&self) -> u8;

    fn update_cartridge(&mut self, cartridge: Cartridge);

    fn check_irq(&self) -> bool {
        false
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MirrorMode {
    MirrorHorizontal,
    MirrorVertical,
    MirrorSingleA,
    MirrorSingleB,
    MirrorFour,
}

pub fn translate_vram(mode: MirrorMode, addr: u16) -> usize {
    (match mode {
        MirrorMode::MirrorHorizontal => (addr & 0x3FF) | ((addr & 0x800) >> 1),
        MirrorMode::MirrorVertical => addr & 0x7FF,
        MirrorMode::MirrorSingleA => addr & 0x3FF,
        MirrorMode::MirrorSingleB => 0x400 | (addr & 0x3FF),
        _ => panic!("Unsupported mirror mode: {:?}", mode),
    }) as usize
}

pub fn make_mapper(cart: Cartridge) -> Box<dyn Mapper> {
    match cart.mapper_id {
        MapperNrom::ID => Box::new(MapperNrom::new(cart)),
        MapperMmc1::ID => Box::new(MapperMmc1::new(cart)),
        MapperMmc3::ID => Box::new(MapperMmc3::new(cart)),
        _ => panic!("Unknown mapper ID: {}", cart.mapper_id),
    }
}

pub fn serialize<S>(m: &Box<dyn Mapper>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut tuple = serializer.serialize_tuple(2)?;
    tuple.serialize_element(&m.get_id())?;
    tuple.serialize_element(m)?;
    tuple.end()
}

struct MapperVisitor;

impl<'de> serde::de::Visitor<'de> for MapperVisitor {
    type Value = Box<dyn Mapper>;

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let id = seq.next_element::<u8>()?.unwrap();
        Ok(match id {
            MapperNrom::ID => Box::new(seq.next_element::<MapperNrom>()?.unwrap()),
            MapperMmc1::ID => Box::new(seq.next_element::<MapperMmc1>()?.unwrap()),
            MapperMmc3::ID => Box::new(seq.next_element::<MapperMmc3>()?.unwrap()),
            _ => panic!("Unknown mapper ID: {}", id),
        })
    }

    fn expecting(&self, _formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        todo!()
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Box<dyn Mapper>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_tuple(2, MapperVisitor)
}
