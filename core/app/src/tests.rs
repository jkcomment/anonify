use super::*;
use init_enclave::EnclaveDir;
use equote::EnclaveContext;
use constants::*;

#[test]
fn test_get_quote() {
    let enclave = EnclaveDir::new().init_enclave().unwrap();
    let enclave_context = EnclaveContext::new(enclave.geteid(), &SPID).unwrap();
    let quote = enclave_context.get_quote().unwrap();
}
