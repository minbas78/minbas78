//! # External "C" functions of the `minbas78` library module.
//!
//! This module declares the public external "C" functions.
//!
use crate::utils;
use rand::{RngExt, SeedableRng, rngs::StdRng};
use regex_lite::Regex;
use std::{
    ffi::{CStr, CString},
    io::{self, Write},
    ops::{Div, Mul},
    os::raw::c_char,
    ptr,
};

/// # pub extern "C" fn input_external.
///
/// External `"C"` function for `INPUT` statement.
///
/// _This external `"C"` function receives a pointer to a string vector and the number of variables declared in the `INPUT` statement._
///
#[unsafe(no_mangle)]
pub extern "C" fn input_external(input_array: *mut *mut c_char, count: u64) {
    print!("{}", utils::INPUT_PREFIX);
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return;
    }
    let mut tokens = input.trim().split(utils::COMMA);
    for i in u64::default()..count {
        let text = match tokens.next() {
            Some(token) => token.trim(),
            None => &String::default(),
        };
        let c_string =
            CString::new(text).unwrap_or_else(|_| CString::new(String::default()).unwrap());
        let raw_pointer = c_string.into_raw();
        unsafe {
            let item_pointer = input_array.offset(i as isize);
            *item_pointer = raw_pointer;
        }
    }
}

/// # pub extern "C" fn print_external.
///
/// External `"C"` function for the `PRINT` statement.
///
/// _This external `"C"` function receives the item to print, the printer column value and if is a `tab_call` grammar rule._
///
#[unsafe(no_mangle)]
pub extern "C" fn print_external(
    print_item: *const c_char,
    print_column: u64,
    is_tab_call_value: bool,
) -> i64 {
    let zone_spaces_total = 15;
    let columns_total = 75;
    let mut stdout = io::stdout();
    unsafe {
        let print_item_c_str = CStr::from_ptr(print_item);
        let print_item_string = print_item_c_str.to_string_lossy().to_string();
        if is_tab_call_value {
            let tab_call = print_item_string.trim().parse::<i64>().unwrap();
            let mut print_column_value = print_column;
            for _ in 1..tab_call {
                let comma_c_string = CString::new(utils::COMMA).unwrap();
                let comma_c_string_pointer = comma_c_string.as_ptr() as *const i8;
                print_column_value =
                    print_external(comma_c_string_pointer, print_column_value, false) as u64;
            }
            return print_column_value as i64;
        }
        let current_item: String;
        let item_length: usize;
        if print_item_string == utils::COMMA {
            let spaces_needed = zone_spaces_total - (print_column % zone_spaces_total);
            current_item = utils::SPACE.repeat(spaces_needed as usize);
            item_length = spaces_needed as usize;
        } else if print_item_string == utils::SEMI_COLON {
            current_item = String::new();
            item_length = usize::default();
        } else if print_item_string == utils::NEW_LINE {
            current_item = print_item_string.clone();
            item_length = usize::default();
        } else {
            current_item = print_item_string.clone();
            item_length = print_item_string.chars().count();
        }
        if print_column + item_length as u64 <= columns_total {
            print!("{}", current_item);
            let _ = stdout.flush();
            return print_column as i64 + item_length as i64;
        }
        let chars: Vec<char> = current_item.chars().collect();
        let space_left = if print_column < columns_total {
            columns_total - print_column
        } else {
            u64::default()
        };
        if space_left > u64::default() {
            let (head, tail) = chars.split_at(space_left as usize);
            let head_string = head.iter().collect::<String>();
            let tail_string = tail.iter().collect::<String>();
            let head_c_string = CString::new(head_string).unwrap();
            let head_c_string_pointer = head_c_string.as_ptr() as *const i8;
            let new_column = print_external(head_c_string_pointer, print_column, is_tab_call_value);
            let tail_c_string = CString::new(tail_string.clone()).unwrap();
            let tail_c_string_pointer = tail_c_string.as_ptr() as *const i8;
            print_external(tail_c_string_pointer, new_column as u64, is_tab_call_value)
        } else {
            print!("{}", utils::NEW_LINE);
            let _ = stdout.flush();
            let actual_item_c_string = CString::new(current_item.clone()).unwrap();
            let actual_item_c_string_pointer = actual_item_c_string.as_ptr() as *const i8;
            print_external(actual_item_c_string_pointer, 0, is_tab_call_value)
        }
    }
}

/// # pub extern "C" fn print_external.
///
/// External `"C"` function for `relational_expression` grammar rule of the `IF` statement.
///
/// _This external `"C"` function receives two strings and returns:_  
/// - `1` if are the same
/// - `0` if are different.
///
#[unsafe(no_mangle)]
pub extern "C" fn strcmp_external(a: *const c_char, b: *const c_char) -> i8 {
    if a.is_null() || b.is_null() {
        return -1;
    }
    unsafe {
        let a_c_str = CStr::from_ptr(a);
        let b_c_str = CStr::from_ptr(b);
        if a_c_str == b_c_str { 1 } else { i8::default() }
    }
}

/// # pub extern "C" fn f64_to_pointer_value_external.
///
/// External `"C"` function for `numeric_expression` grammar rule.
///
/// _This external `"C"` function receives a float number and returns a string representation._  
///
#[unsafe(no_mangle)]
pub extern "C" fn f64_to_pointer_value_external(value: f64, buffer: *const c_char) -> i64 {
    let decimal_maximum_length = 6;
    if buffer.is_null() {
        return i64::default();
    }
    let buffer_mutable = buffer as *mut u8;
    let value_is_negative = value.is_sign_negative();
    let sign_string = if value_is_negative {
        utils::MINUS_SIGN
    } else {
        utils::SPACE
    }
    .to_string();
    let trailing_format = String::from(utils::SPACE);
    let absolute_value = value.abs();
    let mut decimal_point_format = match absolute_value {
        0_f64 => {
            format!("{:<6}", absolute_value)
        }
        1E-5_f64..=9.99999E5 => {
            let absolute_value_string = format!("{}", absolute_value);
            let absolute_value_string_vector = absolute_value_string
                .split(utils::FULL_STOP)
                .collect::<Vec<&str>>();
            let decimal = absolute_value_string_vector
                .get(usize::default())
                .unwrap_or(&utils::ZERO)
                .to_string();
            let decimal_length = decimal.len();
            let mut fraction = absolute_value_string_vector
                .get(1)
                .copied()
                .unwrap_or_default()
                .to_string();
            let fraction_length = fraction.len();
            if decimal_length + fraction_length > decimal_maximum_length {
                format!("{:.5E}", absolute_value)
            } else {
                let mut i = decimal_maximum_length - decimal_length;
                if i > fraction_length {
                    i = fraction_length
                };
                fraction = fraction[..i].to_string();
                format!("{}.{}", decimal, fraction)
            }
        }
        _ => {
            format!("{:.5E}", absolute_value)
        }
    };
    let mut regular_expression_digits = Regex::new(r"0+E").unwrap();
    decimal_point_format = regular_expression_digits
        .replace_all(&decimal_point_format, utils::E)
        .to_string();
    regular_expression_digits = Regex::new(r"\.$").unwrap();
    decimal_point_format = regular_expression_digits
        .replace_all(&decimal_point_format, String::default())
        .to_string();
    let format_value = format!("{}{}{}", sign_string, decimal_point_format, trailing_format);
    let format_value_string = format!("{}", format_value);
    let format_value_bytes = format_value_string.as_bytes();
    let format_value_length = format_value_bytes.len();
    unsafe {
        ptr::copy_nonoverlapping(
            format_value_bytes.as_ptr(),
            buffer_mutable,
            format_value_length,
        );
        buffer_mutable.add(format_value_length).write(b'\0');
    }
    format_value_length as i64
}

/// # pub extern "C" fn pointer_value_to_f64_external.
///
/// External `"C"` function for the `INPUT` statement.
///
/// _This external `"C"` function receives a string and returns the float number representation._  
///
#[unsafe(no_mangle)]
pub extern "C" fn pointer_value_to_f64_external(buffer: *const c_char, value: &mut f64) -> usize {
    if buffer.is_null() {
        return usize::default();
    }
    unsafe {
        let buffer_c_str = CStr::from_ptr(buffer);
        if let Ok(buffer_str) = buffer_c_str.to_str() {
            if let Ok(parsed_value) = buffer_str.parse::<f64>() {
                *value = parsed_value;
                return buffer_str.len();
            }
        }
    }
    usize::default()
}

/// # pub extern "C" fn get_term_external.
///
/// External `"C"` function for `term` grammar rule of the `INPUT` statement.
///
/// _This external `"C"` function receives a pointer to a float number, the len and returns the float value._  
///
#[unsafe(no_mangle)]
pub extern "C" fn get_term_external(pointer: *const f64, length: usize) -> f64 {
    let chunk_size = 2;
    if pointer.is_null() || length == usize::default() {
        return f64::default();
    }
    let slice = unsafe { std::slice::from_raw_parts(pointer, length) };
    let mut stack = slice.to_vec();
    stack.reverse();
    let mut term = stack.pop().unwrap();
    stack.chunks_exact(chunk_size).for_each(|multiplier_chunk| {
        let multiplier = multiplier_chunk.get(usize::default()).unwrap();
        let factor = multiplier_chunk.get(1).unwrap();
        if multiplier.eq(&f64::default()) {
            term = term.mul(factor);
        } else if multiplier.eq(&1_f64) {
            term = term.div(factor);
        }
    });
    term
}

/// # pub extern "C" fn powf_external.
///
/// External `"C"` function for the `POW` built-in function.
///
/// _This external `"C"` function raises a float number to a floating point power._
///
#[unsafe(no_mangle)]
pub extern "C" fn powf_external(base: f64, exponent: f64) -> f64 {
    base.powf(exponent)
}

/// # pub extern "C" fn get_numeric_function_ref_external.
///
/// External `"C"` helper function for the built-in functions.
///
/// _This external `"C"` function manages the call for the built-in functions._
///
#[unsafe(no_mangle)]
pub extern "C" fn get_numeric_function_ref_external(
    numeric_function_name: f64,
    numeric_expression: f64,
) -> f64 {
    match numeric_function_name {
        0_f64 => numeric_expression.abs(),
        1_f64 => atn_external(numeric_expression),
        2_f64 => cos_external(numeric_expression),
        3_f64 => exp_external(numeric_expression),
        4_f64 => numeric_expression.floor(),
        5_f64 => log_external(numeric_expression),
        6_f64 => rnd_external(numeric_expression),
        7_f64 => sgn_external(numeric_expression),
        8_f64 => sin_external(numeric_expression),
        9_f64 => {
            if numeric_expression.is_sign_negative() {
                -1_f64
            } else {
                numeric_expression.sqrt()
            }
        }
        10_f64 => tan_external(numeric_expression),
        99_f64 => 99_f64,
        _ => -1_f64,
    }
}

/// # pub extern "C" fn atn_external.
///
/// External `"C"` function for the `ATN` built-in function.
///
/// _This external `"C"` function computes the arctangent of the float number. The return value is in radians in the range `-pi/2, pi/2`._
///
#[unsafe(no_mangle)]
pub extern "C" fn atn_external(value: f64) -> f64 {
    value.atan()
}

/// # pub extern "C" fn cos_external.
///
/// External `"C"` function for the `COS` built-in function.
///
/// _This external `"C"` function computes the cosine of the float number in radians._
///
#[unsafe(no_mangle)]
pub extern "C" fn cos_external(value: f64) -> f64 {
    value.cos()
}

/// # pub extern "C" fn exp_external.
///
/// External `"C"` function for the `EXP` built-in function.
///
/// _This external `"C"` function returns the exponential function with the formula `e^(float number)`._
///
#[unsafe(no_mangle)]
pub extern "C" fn exp_external(value: f64) -> f64 {
    value.exp()
}

/// # pub extern "C" fn log_external.
///
/// External `"C"` function for the `LOG` built-in function.
///
/// _This external `"C"` function returns the natural logarithm of the float number._
///
#[unsafe(no_mangle)]
pub extern "C" fn log_external(value: f64) -> f64 {
    value.ln()
}

/// # pub extern "C" fn rnd_external.
///
/// External `"C"` function for the `RND` built-in function.
///
/// _This external `"C"` function recives the current seed and returns a random value via the standard uniform distribution._
///
#[unsafe(no_mangle)]
pub extern "C" fn rnd_external(value: f64) -> f64 {
    let value_bits = value.to_bits();
    let mut standard_random_number_generator = StdRng::seed_from_u64(value_bits);
    standard_random_number_generator.random::<f64>()
}

/// # pub extern "C" fn sgn_external.
///
/// External `"C"` function for the `SGN` built-in function.
///
/// _This external `"C"` function returns a number that represents the sign of the float value._  
/// - `1.0` if the number is `positive`, `+0.0` or `INFINITY`  
/// - `-1.0` if the number is `negative`, `-0.0` or `NEG_INFINITY`  
/// - `NaN` if the number is `NaN`.
///
#[unsafe(no_mangle)]
pub extern "C" fn sgn_external(value: f64) -> f64 {
    value.signum()
}

/// # pub extern "C" fn sin_external.
///
/// External `"C"` function for the `SIN` built-in function.
///
/// _This external `"C"` function computes the sine of the float number in radians._
///
#[unsafe(no_mangle)]
pub extern "C" fn sin_external(value: f64) -> f64 {
    value.sin()
}

/// # pub extern "C" fn tan_external.
///
/// External `"C"` function for the `TAN` built-in function.
///
/// _This external `"C"` function computes the tangent of the float number in radians._
///
#[unsafe(no_mangle)]
pub extern "C" fn tan_external(value: f64) -> f64 {
    value.tan()
}
