#![feature(fmt_helpers_for_derive)]

use std::fmt::{Formatter, Result, Debug};

macro_rules! hook {
	($var:ident : $t:ty, $a:expr, $b:expr) => {
		{
			#[allow(non_upper_case_globals)]
			static mut $var: $t = $a;
			static HOOK: $t = $b;
			let detour = retour::RawDetour::new($var as *const (), HOOK as *const ()).unwrap();
			detour.enable().unwrap();
			$var = std::mem::transmute::<*const (), $t>(detour.trampoline());
			std::mem::forget(detour);
		}
	}
}

macro_rules! hook_fmt {
	($color:literal $(,$ty:ty)* $(,)?) => {
		$(hook! {
			func: for<'a, 'b, 'c> fn(&$ty, &'b mut Formatter<'c>) -> Result,
			<$ty as Debug>::fmt,
			|this, fmt| {
				color(fmt, $color)?;
				unsafe { func(this, fmt)? };
				uncolor(fmt)?;
				Ok(())
			}
		})*
	}
}

/// # Safety
/// Must only be called once, probably. Really, not sure.
pub unsafe fn enable() {
	unsafe {
		hook_fmt! { 1, u8, u16, u32, u64, u128 };
		hook_fmt! { 1, i8, i16, i32, i64, i128 };
		hook_fmt! { 1, f32, f64 };
		hook_fmt! { 1, bool };
		hook_fmt! { 2, char };

		hook! {
			hook: for<'b, 'c> fn(&str, &'b mut Formatter<'c>) -> Result,
			<str as Debug>::fmt,
			|this, fmt| {
				color(fmt, 2)?;
				unsafe { hook(this, fmt)? };
				uncolor(fmt)?;
				Ok(())
			}
		}

		hook! {
			hook: for<'a> fn(&'a mut std::fmt::DebugStruct<'static, 'static>, &str, &dyn Debug) -> &'a mut std::fmt::DebugStruct<'static, 'static>,
			std::fmt::DebugStruct::field,
			|fmt, name, val| unsafe { hook(fmt, &colored(name, 5), val) }
		};

		// Coloring struct/tuple names can only be reliably done on nightly, since derived impls have shorthand functions.
		// Unit variants unfortunately use write_str, which I can't hook.
		structs();
		tuples();
	}
}

unsafe fn structs() {
	macro_rules! hook_struct {
		($name:ident $(,$a:ident $b:ident)*) => {
			hook! {
				func: fn(&mut Formatter<'static>, &str $(, $a: &str, $b: &dyn Debug)*) -> Result,
				Formatter::$name,
				|fmt, name $(, $a, $b)*| unsafe { func(fmt, &colored(name, 4) $(, $a, $b)*) }
			}
		}
	}

	hook! {
		hook: for<'b> fn(&'b mut std::fmt::Formatter<'static>, &str) -> std::fmt::DebugStruct<'b, 'static>,
		std::fmt::Formatter::debug_struct,
		|fmt, name| unsafe { hook(fmt, &colored(name, 4)) }
	};

	hook_struct!(debug_struct_field1_finish, n1 v1);
	hook_struct!(debug_struct_field2_finish, n1 v1, n2 v2);
	hook_struct!(debug_struct_field3_finish, n1 v1, n2 v2, n3 v3);
	hook_struct!(debug_struct_field4_finish, n1 v1, n2 v2, n3 v3, n4 v4);
	hook_struct!(debug_struct_field5_finish, n1 v1, n2 v2, n3 v3, n4 v4, n5 v5);
}

unsafe fn tuples() {
	macro_rules! hook_tuple {
		($name:ident $(,$a:ident)*) => {
			hook! {
				func: fn(&mut Formatter<'static>, &str $(, $a: &dyn Debug)*) -> Result,
				Formatter::$name,
				|fmt, name $(, $a)*| unsafe { func(fmt, &colored(name, 4) $(, $a)*) }
			}
		}
	}

	hook! {
		hook: for<'b> fn(&'b mut std::fmt::Formatter<'static>, &str) -> std::fmt::DebugTuple<'b, 'static>,
		std::fmt::Formatter::debug_tuple,
		|fmt, name| unsafe { hook(fmt, &colored(name, 4)) }
	};

	hook_tuple!(debug_tuple_field1_finish, v1);
	hook_tuple!(debug_tuple_field2_finish, v1, v2);
	hook_tuple!(debug_tuple_field3_finish, v1, v2, v3);
	hook_tuple!(debug_tuple_field4_finish, v1, v2, v3, v4);
	hook_tuple!(debug_tuple_field5_finish, v1, v2, v3, v4, v5);
}

fn color(fmt: &mut Formatter, color: u8) -> Result {
	write!(fmt, "\x1b[3{}m", color)
}

fn uncolor(fmt: &mut Formatter) -> Result {
	write!(fmt, "\x1b[39m")
}

fn colored(name: &str, color: u8) -> String {
	format!("\x1b[3{}m{}\x1b[39m", color, name)
}

#[test]
fn test() {
	unsafe {
		enable();
	}

	#[derive(Debug)]
	struct Nested {
		name: String,
		age: i32,
		things: Vec<String>,
		other: Option<Box<Nested>>,
	}

	let val = Nested {
		name: "Alice".to_string(),
		age: 42,
		things: vec!["one".to_string(), "two".to_string()],
		other: Some(Box::new(Nested {
			name: "Bob".to_string(),
			age: 24,
			things: vec!["three".to_string(), "four".to_string()],
			other: None,
		})),
	};

	println!("{val:#?}");
}
