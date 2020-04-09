
#![allow(dead_code)]
#![allow(unused_imports)]

use move_vm_runtime::{
    MoveVM,
};
use move_vm_state::{
    //data_cache::{BlockDataCache, RemoteCache},
    execution_context::{ExecutionContext, SystemExecutionContext},
};
//use bytecode_source_map::source_map::SourceMap;
use bytecode_verifier::{
    verifier::{VerifiedScript,VerifiedModule}
};
use language_e2e_tests::{
    account::{Account, AccountData},
    data_store::FakeDataStore,
};
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
    write_set::{WriteSet, WriteOp},
};
use compiler::Compiler;
use std::{
    path::{Path, PathBuf},
    fs,
    io::Write,
};
use stdlib::{stdlib_modules, StdLibOptions};
use move_vm_types::values::Value;
use include_dir::{include_dir, Dir};

const PRELOAD_DIR: Dir = include_dir!("./src/modules");

fn main() {

    let address = account_config::association_address(); //AccountAddress::default();
    //let para1 = Value::address(address);
    let args = vec![];   
    let source_path = Path::new("/Users/liangping/workspace/hello/src/scripts/test.mvir");
    let _mv_extension = "mv";
    let sm_extension = "mvsm";

    println!("{:?}", address); 

    // prepare for startup.
    let mut data_cache: FakeDataStore = FakeDataStore::default();

    let mut stdlib = stdlib_modules(StdLibOptions::Fresh).to_vec();
    for x in &stdlib {
        let cm = &x.as_inner();
        data_cache.add_module(&cm.self_id(), cm);
    };
    let pattern = "**/*.mvir";
    for entry in PRELOAD_DIR.find(pattern).unwrap() {
        println!("preload: {}", entry.path().display());
        let m: VerifiedModule = pre_complie(entry.path(), address);

        let cm = &m.as_inner();
        data_cache.add_module(&cm.self_id(), cm);
        stdlib.push(m);
    };

    let acc = Account::new_association(); // Account::new_genesis_account(address);
    let account_data = AccountData::with_account(acc, 10000000000, 1);
    data_cache.add_account_data( &account_data );

    // Compile script: 
    let compiler = Compiler {
        address,
        skip_stdlib_deps: false,
        extra_deps: stdlib.clone(),
        ..Compiler::default()
    };

    let source = fs::read_to_string(source_path.as_os_str()).expect("Unable to read file");

    let (compiled_program, source_map) = compiler.into_compiled_script_and_source_map(source_path.as_os_str().to_str().unwrap(), &source)
            .expect("Failed to compile program");
    let verified_program = VerifiedScript::new(compiled_program)
            .expect("Failed to verify program");
    let compiled_program = verified_program.into_inner();

    let mut script: Vec<u8> = vec![];
    compiled_program.as_inner()
        .serialize(&mut script)
        .expect("Unable to serialize script"); 

    if cfg!(source_map) {
        let source_map_bytes = serde_json::to_vec(&source_map).expect("Unable to serialize program");
        write_output(&source_path.with_extension(sm_extension), &source_map_bytes);   
    }
    // Execute script. 
    // create a Move VM and populate it with generated modules
    let move_vm = MoveVM::new();
    let mut ctx = SystemExecutionContext::new(&data_cache, GasUnits::new(0));
    let gas_schedule = CostTable::zero();


    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = address;

    let result: VMResult<()> = move_vm.execute_script(script, &gas_schedule, &mut ctx, &txn_data, vec![],args);
    
    println!("output from move vm: {:?}",  result);

    let ws = ctx.make_write_set().unwrap();
    println!("{},=>", &ws.len());

    for (a, wo) in ws {
        println!("path:{}, {:?}", a, wo);
        match wo {
            WriteOp::Deletion=> println!("delete"),
            WriteOp::Value(v)=> println!("{:?}", v),
        }
    }

}

fn write_output(path: &PathBuf, buf: &[u8]) {
    let mut f = fs::File::create(path).expect("Error occurs on create output file");
    f.write_all(&buf).expect("Error occurs on writing output file");
}

fn pre_complie( path: &Path, address: AccountAddress) -> VerifiedModule {
    // Compile script: 
    let compiler = Compiler {
        //address: AccountAddress::from_hex_literal("0xc4ece0edf07cfbf7fae8714f668ecdf1").unwrap(),
        address: address,
        skip_stdlib_deps: false,
        extra_deps: stdlib_modules(StdLibOptions::Staged).to_vec(),
        ..Compiler::default()
    };

    let source = PRELOAD_DIR.get_file(path.to_str().unwrap()).unwrap().contents_utf8().unwrap();

    let compiled_module = compiler.into_compiled_module(path.as_os_str().to_str().unwrap(), &source)
            .expect("Failed to compile module");

    VerifiedModule::bypass_verifier_DANGEROUS_FOR_TESTING_ONLY(compiled_module)
}

// fn fetch_gas_schedule(&mut self, data_cache: &dyn RemoteCache) -> VMResult<CostTable> {
//     let address = account_config::association_address();
//     let mut ctx = SystemExecutionContext::new(data_cache, GasUnits::new(0));
//     let gas_struct_ty = self
//         .move_vm
//         .resolve_struct_def_by_name(&GAS_SCHEDULE_MODULE, &GAS_SCHEDULE_NAME, &mut ctx, &[])
//         .map_err(|_| {
//             VMStatus::new(StatusCode::GAS_SCHEDULE_ERROR)
//                 .with_sub_status(sub_status::GSE_UNABLE_TO_LOAD_MODULE)
//         })?;

//     let access_path = create_access_path(address, gas_struct_ty.into_struct_tag()?);

//     let data_blob = data_cache
//         .get(&access_path)
//         .map_err(|_| {
//             VMStatus::new(StatusCode::GAS_SCHEDULE_ERROR)
//                 .with_sub_status(sub_status::GSE_UNABLE_TO_LOAD_RESOURCE)
//         })?
//         .ok_or_else(|| {
//             VMStatus::new(StatusCode::GAS_SCHEDULE_ERROR)
//                 .with_sub_status(sub_status::GSE_UNABLE_TO_LOAD_RESOURCE)
//         })?;
//     let table: CostTable = lcs::from_bytes(&data_blob).map_err(|_| {
//         VMStatus::new(StatusCode::GAS_SCHEDULE_ERROR)
//             .with_sub_status(sub_status::GSE_UNABLE_TO_DESERIALIZE)
//     })?;

//     Ok(table)
// }
