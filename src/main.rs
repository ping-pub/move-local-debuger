
#![allow(dead_code)]

use move_vm_runtime::{
    MoveVM,
};
use move_vm_state::{
    //data_cache::{BlockDataCache, RemoteCache},
    execution_context::{SystemExecutionContext},
};
use bytecode_verifier::{
    verifier::{
        //verify_module_dependencies, 
        VerifiedProgram},
    //VerifiedModule,
};
use language_e2e_tests::data_store::FakeDataStore;
use vm::{
    errors::VMResult,
    //access::ModuleAccess,
    gas_schedule::{
        //AbstractMemorySize, 
        CostTable, GasAlgebra,
        //GasCarrier, 
        GasUnits},
    transaction_metadata::TransactionMetadata,
};
use libra_types::{
    account_address::AccountAddress,
    account_config,
    transaction::{
        //Module, 
        Script,
        TransactionArgument,
    },
};
use compiler::Compiler;
use std::{
    path::{Path, PathBuf},
    fs,
    io::Write,
};
use stdlib::{stdlib_modules, StdLibOptions};
use move_vm_types::values::Value;

fn main() {

    let address = account_config::association_address(); //AccountAddress::default();
    let para1 = Value::address(address);
    let args = vec![para1];   
    let source_path = Path::new("/Users/liangping/workspace/hello/src/scripts/add_validator.mvir");
    let mv_extension = "mv";

    println!("{:?}", address); 
    
    // Compile script: 
    let compiler = Compiler {
        address,
        skip_stdlib_deps: false,
        extra_deps: stdlib_modules(StdLibOptions::Staged).to_vec(),
        ..Compiler::default()
    };

    let source = fs::read_to_string(source_path.as_os_str()).expect("Unable to read file");

    let (compiled_program, _, dependencies) = compiler.into_compiled_program_and_source_maps_deps(source_path.as_os_str().to_str().unwrap(), &source)
            .expect("Failed to compile program");
    let verified_program = VerifiedProgram::new(compiled_program, &dependencies)
            .expect("Failed to verify program");
    let compiled_program = verified_program.into_inner();

    let mut script: Vec<u8> = vec![];
    compiled_program
        .script
        .serialize(&mut script)
        .expect("Unable to serialize script");
    let payload = Script::new(script.clone(), vec![]);
    let payload_bytes = serde_json::to_vec(&payload).expect("Unable to serialize program");
    write_output(&source_path.with_extension(mv_extension), &payload_bytes);        
    
    // Execute script. 
    // create a Move VM and populate it with generated modules
    let move_vm = MoveVM::new();
    let data_cache = FakeDataStore::default();
    let mut ctx = SystemExecutionContext::new(&data_cache, GasUnits::new(0));
    let gas_schedule = CostTable::zero();

    // load std modules
    let mut txn_stdlib = TransactionMetadata::default();
    txn_stdlib.sender = account_config::CORE_CODE_ADDRESS;
    let std = stdlib_modules(StdLibOptions::Staged).iter();
    for x in std {
        let mut bytes:Vec<u8> = vec![];
        x.serialize(&mut bytes).expect("Std Modules serialize failed.");
        move_vm.publish_module(bytes, &mut ctx, &txn_stdlib).expect("Publish failed"); 
    };
     
    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = address;
    let result: VMResult<()> = move_vm.execute_script(script, &gas_schedule, &mut ctx, &txn_data, args);
    
    println!("output from move vm: {:?}",  result);

}

fn write_output(path: &PathBuf, buf: &[u8]) {
    let mut f = fs::File::create(path).expect("Error occurs on create output file");
    f.write_all(&buf).expect("Error occurs on writing output file");
}
