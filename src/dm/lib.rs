#![feature(type_ascription)]

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use context::DMContext;
pub use dm_impl;
use global_state::GLOBAL_STATE;
use value::EitherValue;
use value::Value;

mod byond_ffi;
mod context;
mod global_state;
mod hooks;
mod proc;
mod raw_types;
mod string;
mod value;

macro_rules! signature {
	($sig:tt) => {
		$crate::dm_impl::convert_signature!($sig)
	};
}

fn random_string(n: usize) -> String {
	thread_rng().sample_iter(&Alphanumeric).take(n).collect()
}

// signature!("3D ?? ?? ?? ?? 74 14 50 E8 ?? ?? ?? ?? FF 75 0C FF 75 08 E8")
// should hopefully work when someone bothers
static SIGNATURES: phf::Map<&'static str, Vec<Option<u8>>> = phf::phf_map! {
	"string_table" => signature!("A1 ?? ?? ?? ?? 8B 04 ?? 85 C0 0F 84 ?? ?? ?? ?? 80 3D ?? ?? ?? ?? 00 8B 18 "),
	"get_proc_array_entry" => signature!("E8 ?? ?? ?? ?? 8B C8 8D 45 ?? 6A 01 50 FF 76 ?? 8A 46 ?? FF 76 ?? FE C0"),
	"get_string_id" => signature!("55 8B EC 8B 45 ?? 83 EC ?? 53 56 8B 35"),
	"call_proc_by_id" => signature!("55 8B EC 81 EC ?? ?? ?? ?? A1 ?? ?? ?? ?? 33 C5 89 45 ?? 8B 55 ?? 8B 45"),
	"get_variable" => signature!("55 8B EC 8B 4D ?? 0F B6 C1 48 83 F8 ?? 0F 87 ?? ?? ?? ?? 0F B6 80 ?? ?? ?? ?? FF 24 85 ?? ?? ?? ?? FF 75 ?? FF 75 ?? E8"),
	"set_variable" => signature!("55 8B EC 8B 4D 08 0F B6 C1 48 57 8B 7D 10 83 F8 53 0F ?? ?? ?? ?? ?? 0F B6 80 ?? ?? ?? ?? FF 24 85 ?? ?? ?? ?? FF 75 18 FF 75 14 57 FF 75 0C E8 ?? ?? ?? ?? 83 C4 10 5F 5D C3"),
	"get_string_table_entry" => signature!("55 8B EC 8B 4D 08 3B 0D ?? ?? ?? ?? 73 10 A1"),
	"call_datum_proc_by_name" => signature!("55 8B EC 83 EC 0C 53 8B 5D 10 8D 45 FF 56 8B 75 14 57 6A 01 50 FF 75 1C C6 45 FF 00 FF 75 18 6A 00 56 53 "),
	"dec_ref_count" => signature!("3D ?? ?? ?? ?? 74 14 50 E8 ?? ?? ?? ?? FF 75 0C FF 75 08 E8 "),
	"inc_ref_count" => signature!("FF 75 10 E8 ?? ?? ?? ?? FF 75 0C 8B F8 FF 75 08 E8 ?? ?? ?? ?? 57 "),
};

byond_ffi_fn! { auxtools_init(_input) {
	// Already initialized. Just succeed?
	if GLOBAL_STATE.get().is_some() {
		return Some("SUCCESS".to_owned());
	}

	let x: Vec<Option<u8>> = signature!("3D ?? ?? ?? ?? 74 14 50 E8 ?? ?? ?? ?? FF 75 0C FF 75 08 E8");

	let byondcore = match sigscan::Scanner::for_module("byondcore.dll") {
		Some(v) => v,
		None => return Some("FAILED (Couldn't create scanner for byondcore.dll)".to_owned())
	};

	let string_table: *mut raw_types::strings::StringTable;
	if let Some(ptr) = byondcore.find(SIGNATURES.get("string_table").unwrap()) {
		unsafe {
			// TODO: Could be nulls
			string_table = *(ptr.offset(1) as *mut *mut raw_types::strings::StringTable);
		}
	} else {
		return Some("FAILED (Couldn't find stringtable)".to_owned())
	}

	let get_proc_array_entry: raw_types::funcs::GetProcArrayEntry;
	if let Some(ptr) = byondcore.find(SIGNATURES.get("get_proc_array_entry").unwrap()) {
		unsafe {
			// TODO: Could be nulls
			let offset = *(ptr.offset(1) as *const isize);
			get_proc_array_entry = std::mem::transmute(ptr.offset(5).offset(offset) as *const ());
		}
	} else {
		return Some("FAILED (Couldn't find GetProcArrayEntry)".to_owned())
	}

	let get_string_id: raw_types::funcs::GetStringId;
	if let Some(ptr) = byondcore.find(SIGNATURES.get("get_string_id").unwrap()) {
		unsafe {
			// TODO: Could be nulls
			get_string_id = std::mem::transmute(ptr as *const ());
		}
	} else {
		return Some("FAILED (Couldn't find GetStringId)".to_owned())
	}

	let call_proc_by_id: raw_types::funcs::CallProcById;
	if let Some(ptr) = byondcore.find(SIGNATURES.get("call_proc_by_id").unwrap()) {
		unsafe {
			// TODO: Could be nulls
			call_proc_by_id = std::mem::transmute(ptr as *const ());
		}
	} else {
		return Some("FAILED (Couldn't find CallGlobalProc)".to_owned())
	}

	let get_variable: raw_types::funcs::GetVariable;
	if let Some(ptr) = byondcore.find(SIGNATURES.get("get_variable").unwrap()) {
		unsafe {
			// TODO: Could be nulls
			get_variable = std::mem::transmute(ptr as *const ());
		}
	} else {
		return Some("FAILED (Couldn't find GetVariable)".to_owned())
	}

	let set_variable: raw_types::funcs::SetVariable;
	if let Some(ptr) = byondcore.find(SIGNATURES.get("set_variable").unwrap()) {
		unsafe {
			// TODO: Could be nulls
			set_variable = std::mem::transmute(ptr as *const ());
		}
	} else {
		return Some("FAILED (Couldn't find SetVariable)".to_owned())
	}

	let get_string_table_entry: raw_types::funcs::GetStringTableEntry;
	if let Some(ptr) = byondcore.find(SIGNATURES.get("get_string_table_entry").unwrap()) {
		unsafe {
			// TODO: Could be nulls
			get_string_table_entry = std::mem::transmute(ptr as *const ());
		}
	} else {
		return Some("FAILED (Couldn't find GetStringTableEntry)".to_owned())
	}

	let call_datum_proc_by_name: raw_types::funcs::CallDatumProcByName;
	if let Some(ptr) = byondcore.find(SIGNATURES.get("call_datum_proc_by_name").unwrap()) {
		unsafe {
			// TODO: Could be nulls
			call_datum_proc_by_name = std::mem::transmute(ptr as *const ());
		}
	} else {
		return Some("FAILED (Couldn't find CallDatumProcByName)".to_owned())
	}

	/*
	char* x_ref_count_call = (char*)Pocket::Sigscan::FindPattern(BYONDCORE, "3D ?? ?? ?? ?? 74 14 50 E8 ?? ?? ?? ?? FF 75 0C FF 75 08 E8", 20);
	DecRefCount = (DecRefCountPtr)(x_ref_count_call + *(int*)x_ref_count_call + 4); //x_ref_count_call points to the relative offset to DecRefCount from the call site
	x_ref_count_call = (char*)Pocket::Sigscan::FindPattern(BYONDCORE, "FF 75 10 E8 ?? ?? ?? ?? FF 75 0C 8B F8 FF 75 08 E8 ?? ?? ?? ?? 57", 17);
	IncRefCount = (IncRefCountPtr)(x_ref_count_call + *(int*)x_ref_count_call + 4);
	*/
	let dec_ref_count: raw_types::funcs::DecRefCount;
	let inc_ref_count: raw_types::funcs::IncRefCount;
	unsafe {
		let dec_ref_count_call: *const u8 = byondcore.find(SIGNATURES.get("dec_ref_count_call").unwrap()).unwrap().offset(20);
		dec_ref_count = std::mem::transmute(dec_ref_count_call.offset((*(dec_ref_count_call as *const u32) + 4) as isize));

		let inc_ref_count_call: *const u8 = byondcore.find(SIGNATURES.get("inc_ref_count_call").unwrap()).unwrap().offset(17);
		inc_ref_count = std::mem::transmute(inc_ref_count_call.offset((*(inc_ref_count_call as *const u32) + 4) as isize));
	}

	if GLOBAL_STATE.set(global_state::State {
		get_proc_array_entry: get_proc_array_entry,
		get_string_id: get_string_id,
		execution_context: std::ptr::null_mut(),
		string_table: string_table,
		call_proc_by_id: call_proc_by_id,
		get_variable: get_variable,
		set_variable: set_variable,
		get_string_table_entry: get_string_table_entry,
		call_datum_proc_by_name: call_datum_proc_by_name,
		dec_ref_count: dec_ref_count,
		inc_ref_count: inc_ref_count

	}).is_err() {
		panic!();
	}

	if let Err(error) = hooks::init() {
		return Some(error);
	}

	proc::populate_procs();

	hooks::hook("/proc/wew", hello_proc_hook).unwrap_or_else(|e| {
			msgbox::create("Failed to hook!", e.to_string().as_str(), msgbox::IconType::Error)
		}
	);

	Some("SUCCESS".to_owned())
} }

macro_rules! args {
    () => {
        None
    };
    ($($x:expr),+ $(,)?) => {
        Some(vec![$(value::EitherValue::from($x),)+])
    };
}

fn hello_proc_hook<'a>(
	ctx: &'a DMContext,
	src: Value<'a>,
	usr: Value<'a>,
	args: &Vec<Value<'a>>,
) -> EitherValue<'a> {
	let dat = args[0];

	let string: string::StringRef = "penis".into();
	let string2: string::StringRef = "penisaaa".into();

	string.into()
}

#[cfg(test)]
mod tests {
	#[test]
	fn test() {}
}
