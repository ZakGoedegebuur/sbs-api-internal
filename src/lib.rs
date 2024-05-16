use std::{error, fs, path, mem};

pub trait Serialize {
    fn serialize(&self, sbi: &mut SBI);
} 

pub trait DeSerialize {
    fn deserialize(sbi: &mut SBI, offset: &mut usize) -> Result<Self, ()> where Self: Sized;
}

pub struct SBI {
    pub data: Vec<u8>,
}

impl SBI {
    pub fn new() -> Self {
        Self {
            data: Vec::new()
        }
    }

    pub fn from_path<P: AsRef<path::Path>>(path: P) -> Result<Self, Box<dyn error::Error>> {
        let file = fs::read(path)?;

        Ok(Self {
            data: file,
        })
    }
    
    pub fn deserialize<T: DeSerialize>(&mut self) -> Result<T, ()> {
        let mut offset = 0;
        T::deserialize(self, &mut offset)
    }

    pub fn serialize<T: Serialize>(&mut self, root: T) {
        root.serialize(self)
    }

    pub fn write_to_path<P: AsRef<path::Path>>(&self, path: P) -> Result<(), Box<dyn error::Error>> {
        fs::write(path, &self.data)?;
        Ok(())
    }
}

macro_rules! impl_serde_for_num {
    ($($t:ty),*) => {
        $(
            impl Serialize for $t {
                fn serialize(&self, sbi: &mut SBI) {
                    sbi.data.extend_from_slice(&self.to_be_bytes());
                }
            }

            impl DeSerialize for $t {
                fn deserialize(sbi: &mut SBI, offset: &mut usize) -> Result<Self, ()> {
                    const SIZE: usize = mem::size_of::<$t>();
                    let end_offset = *offset + SIZE;

                    if end_offset > sbi.data.len() {
                        Err(())
                    } else {
                        let data: [u8; SIZE] = (&sbi.data[*offset..end_offset]).try_into().unwrap();
                        *offset = end_offset;
                        Ok(<$t>::from_be_bytes(data))
                    }
                }
            }
        )*
    };
}

impl_serde_for_num!(
    i8, 
    i16, 
    i32, 
    i64, 
    i128, 
    isize, 
    u8, 
    u16, 
    u32, 
    u64, 
    u128, 
    usize,
    f32,
    f64
);

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self, sbi: &mut SBI) {
        (self.len() as u64).serialize(sbi);
    
        for item in self.iter() {
            item.serialize(sbi)
        }
    }
}

impl<T: DeSerialize> DeSerialize for Vec<T> {
    fn deserialize(sbi: &mut SBI, offset: &mut usize) -> Result<Self, ()> where Self: Sized {
        let len = u64::deserialize(sbi, offset)?;

        let mut ret = Vec::with_capacity(len as usize);
        for _ in 0..len {
            ret.push(T::deserialize(sbi, offset)?);
        }

        Ok(ret)
    }
}

impl Serialize for String {
    fn serialize(&self, sbi: &mut SBI) {
        (self.len() as u64).serialize(sbi);
        
        sbi.data.extend_from_slice(self.as_bytes())
    }
}

impl DeSerialize for String {
    fn deserialize(sbi: &mut SBI, offset: &mut usize) -> Result<Self, ()> where Self: Sized {
        let len = u64::deserialize(sbi, offset)?;

        let string = String::from_utf8_lossy(&sbi.data[*offset..*offset + len as usize]).to_string();
        *offset += len as usize;

        Ok(string)
    }
}