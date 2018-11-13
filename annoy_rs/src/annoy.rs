use super::native::*;

pub struct AnnoyLib {
    raw: rust_annoy_index_t,
}

impl AnnoyLib {
    pub fn new(f: i32) -> AnnoyLib {
        let raw = unsafe { rust_annoy_index_angular_init(f) };
        AnnoyLib { raw }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build() {
        let _a = AnnoyLib::new(13);
    }
}
