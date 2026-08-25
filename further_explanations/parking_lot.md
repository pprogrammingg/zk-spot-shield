Zero-Copy
- how do bytemuck::Pod and bytemuck::Zeroable work? why we have padding requirements?
    - Rust default struct layout has arbitrary padding. #[repr(C)] ensures predictable C-style field ordering, and explicit padding (_padding: [u8; 7]) ensures the struct size is a strict multiple of 8 bytes (64-bit alignment) with no uninitialized implicit gaps.

    
ZK
- In this project where does prover and verifier sit?
