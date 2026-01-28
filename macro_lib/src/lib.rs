#[macro_export]
macro_rules! type_trans {
    ($ptr:ident, $ty:ty) => {
        $ptr as *const $ty
    };
}

#[macro_export]
macro_rules! value_operation {
    ($left:ident, $binop:tt, $right:ident) => {
        unsafe { {
            use VarType::*;
        let left_type = (*$left).ty.clone();
        let right_type = (*$right).ty.clone();

        match (left_type, right_type) {
            (Int, Int) => {
                let left=type_trans!($left, IntObject);
                let right=type_trans!($right, IntObject);

                let left_val=(*left).value;
                let right_val=(*right).value;

                let res=alloc_int((left_val $binop right_val));
                return res
            },
            (Int, Float) => {
                let left=type_trans!($left, IntObject);
                let right=type_trans!($right, FloatObject);

                let left_val=(*left).value as f64;
                let right_val=(*right).value;

                let res=alloc_float(left_val $binop right_val);
                return res
            },
            (Int, Bool) => {
                let left=type_trans!($left, IntObject);
                let right=type_trans!($right, BoolObject);

                let left_val=(*left).value;
                let right_val=match (*right).value{
                    true=>1,
                    false=>0
                };

                let res=alloc_int(left_val $binop right_val);
                return res
            },
            (Int, Char) => {
                let left=type_trans!($left, IntObject);
                let right=type_trans!($right, CharObject);

                let left_val=(*left).value;
                let right_val=(*right).value;

                let res=alloc_char(left_val as u8 $binop right_val);
                return res
            },
            (Int, String) => {
                let left=type_trans!($left, IntObject);
                let right =type_trans!($right, StringObject);

                let left_val=(*left).value;
                let right_val=(*right).data;
                let len=(*right).len;

                let left_str=left_val.to_string();

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(len + left_str.len()) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(
                    left_str.as_ptr(), 
                    new_ptr, 
                    left_str.len()
                );
                std::ptr::copy_nonoverlapping(
                    right_val, 
                    new_ptr.add(left_str.len()), 
                    len
                );

                let res=alloc_string(new_ptr, len+left_str.len());
                return res
            },
            (Float, Int) => {
                let $left=type_trans!($left, FloatObject);
                let $right=type_trans!($right, IntObject);

                let left_val=(*$left).value;
                let right_val=(*$right).value as f64;

                let res=alloc_float(left_val $binop right_val);
                return res
            },
            (Float, Float) => {
                let $left=type_trans!($left, FloatObject);
                let $right=type_trans!($right, FloatObject);

                let left_val=(*$left).value;
                let right_val=(*$right).value;

                let res=alloc_float(left_val as f64 $binop right_val);
                return res
            },
            (Float, Bool) => {
                let $left=type_trans!($left, FloatObject);
                let $right=type_trans!($right, BoolObject);

                let left_val=(*$left).value;
                let right_val=match (*$right).value{
                    true=>1.0,
                    false=>0.0
                };

                let res=alloc_float(left_val as f64 $binop right_val);
                return res
            },
            (Float, Char) => {
                let left=type_trans!($left, FloatObject);
                let right=type_trans!($right, CharObject);

                let left_val=(*left).value;
                let right_val=(*right).value as f64;

                let res=alloc_float(left_val $binop right_val);
                return res
            },
            (Float, String) => {
                let left=type_trans!($left, FloatObject);
                let right =type_trans!($right, StringObject);

                let left_val=(*left).value;
                let right_val=(*right).data;
                let len=(*right).len;

                let left_str=left_val.to_string();

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(len + left_str.len()) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(
                    left_str.as_ptr(), 
                    new_ptr, 
                    left_str.len()
                );
                std::ptr::copy_nonoverlapping(
                    right_val, 
                    new_ptr.add(left_str.len()), 
                    len
                );

                let res=alloc_string(new_ptr, len+left_str.len());
                return res
            },
            (Bool, Int) => {
                let left=type_trans!($left, BoolObject);
                let right=type_trans!($right, IntObject);

                let left_val=match (*left).value{
                    true=>1,
                    false=>0
                };
                let right_val=(*right).value;

                let res=alloc_int(left_val $binop right_val);
                return res
            },
            (Bool, Float) => {
                let $left=type_trans!($left, BoolObject);
                let $right=type_trans!($right, FloatObject);

                let left_val=match (*$left).value{
                    true=>1.0,
                    false=>0.0
                };
                let right_val=(*$right).value;

                let res=alloc_float(left_val $binop right_val);
                return res
            },
            (Bool, Bool) => {
                let $left=type_trans!($left, BoolObject);
                let $right=type_trans!($right, BoolObject);

                let left_val=match (*$left).value{
                    true=>1,
                    false=>0
                };
                let right_val=match (*$right).value{
                    true=>1,
                    false=>0
                };

                let res=alloc_int(left_val $binop right_val);
                return res
            },
            (Bool, Char) => {
                let $left=type_trans!($left, BoolObject);
                let $right=type_trans!($right, CharObject);

                let left_val=match (*$left).value{
                    true=>1,
                    false=>0
                };
                let right_val=(*$right).value;

                let res=alloc_char(left_val $binop right_val);
                return res
            },
            (Bool, String) => {
                let left=type_trans!($left, BoolObject);
                let right =type_trans!($right, StringObject);

                let left_val=(*left).value;
                let right_val=(*right).data;
                let len=(*right).len;

                let left_str=left_val.to_string();

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(len + left_str.len()) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(
                    left_str.as_ptr(), 
                    new_ptr, 
                    left_str.len()
                );
                std::ptr::copy_nonoverlapping(
                    right_val, 
                    new_ptr.add(left_str.len()), 
                    len
                );

                let res=alloc_string(new_ptr, len+left_str.len());
                return res
            },
            (Char, Int) => {
                let left=type_trans!($left, CharObject);
                let right=type_trans!($right, IntObject);

                let left_val=(*left).value;
                let right_val=(*right).value;

                let res=alloc_char(left_val $binop right_val as u8);
                return res
            },
            (Char, Float) => {
                let left=type_trans!($left, CharObject);
                let right=type_trans!($right, FloatObject);

                let left_val=(*left).value as f64;
                let right_val=(*right).value;

                let res=alloc_float(left_val $binop right_val);
                return res
            },
            (Char, Bool) => {
                let $left=type_trans!($left, CharObject);
                let $right=type_trans!($right, BoolObject);

                let left_val=(*$left).value;
                let right_val=match (*$right).value{
                    true=>1,
                    false=>0
                };

                let res=alloc_char(left_val $binop right_val);
                return res
            },
            (Char, Char) => {
                let $left=type_trans!($left, CharObject);
                let $right=type_trans!($right, CharObject);

                let left_val=(*$left).value;
                let right_val=(*$right).value;

                let res=alloc_char(left_val $binop right_val);
                return res
            },
            (Char, String) => {
                let left=type_trans!($left, CharObject);
                let right =type_trans!($right, StringObject);

                let left_val=(*left).value;
                let right_val=(*right).data;
                let len=(*right).len;

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(len + 1) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 2. 写入新字符
                std::ptr::write(new_ptr, left_val);

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(right_val, new_ptr.add(1), len);

                let res=alloc_string(new_ptr, len+1);
                return res
            },
            (String, Int) => {
                let left=type_trans!($left, StringObject);
                let right =type_trans!($right, IntObject);

                let right_val=(*right).value;
                let left_val=(*left).data;
                let len=(*left).len;

                let right_str=right_val.to_string();

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(len + right_str.len()) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(
                    left_val, 
                    new_ptr, 
                    len
                );
                std::ptr::copy_nonoverlapping(
                    right_str.as_ptr(), 
                    new_ptr.add(len), 
                    right_str.len()
                );

                let res=alloc_string(new_ptr, len+right_str.len());
                return res
            },
            (String, Float) => {
                let left=type_trans!($left, StringObject);
                let right =type_trans!($right, FloatObject);

                let right_val=(*right).value;
                let left_val=(*left).data;
                let len=(*left).len;

                let right_str=right_val.to_string();

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(len + right_str.len()) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(
                    left_val, 
                    new_ptr, 
                    len
                );
                std::ptr::copy_nonoverlapping(
                    right_str.as_ptr(), 
                    new_ptr.add(len), 
                    right_str.len()
                );

                let res=alloc_string(new_ptr, len+right_str.len());
                return res
            },
            (String, Bool) => {
                let left=type_trans!($left, StringObject);
                let right =type_trans!($right, BoolObject);

                let right_val=(*right).value;
                let left_val=(*left).data;
                let len=(*left).len;

                let right_str=right_val.to_string();

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(len + right_str.len()) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(
                    left_val, 
                    new_ptr, 
                    len
                );
                std::ptr::copy_nonoverlapping(
                    right_str.as_ptr(), 
                    new_ptr.add(len), 
                    right_str.len()
                );

                let res=alloc_string(new_ptr, len+right_str.len());
                return res
            },
            (String, Char) => {
                let left=type_trans!($left, StringObject);
                let right =type_trans!($right, CharObject);

                let right_val=(*right).value;
                let left_val=(*left).data;
                let len=(*left).len;

                let right_str=right_val.to_string();

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(len + right_str.len()) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(
                    left_val, 
                    new_ptr, 
                    len
                );
                std::ptr::copy_nonoverlapping(
                    right_str.as_ptr(), 
                    new_ptr.add(len), 
                    right_str.len()
                );

                let res=alloc_string(new_ptr, len+right_str.len());
                return res
            },
            (String, String) => {
                let left=type_trans!($left, StringObject);
                let right =type_trans!($right, StringObject);

                let left_val=(*left).data;
                let left_len=(*left).len;
                let right_val=(*right).data;
                let right_len=(*right).len;

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(left_len + right_len) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(
                    left_val, 
                    new_ptr, 
                    left_len
                );
                std::ptr::copy_nonoverlapping(
                    right_val, 
                    new_ptr.add(left_len), 
                    right_len
                );

                let res=alloc_string(new_ptr, left_len+right_len);
                return res
            },
            (Unknown, _) => {
                let msg="unknown type";
                let obj=alloc_string(msg.as_ptr(), msg.len());
                return std::ptr::null_mut() as *mut Object
            },
            (_,Unknown) => {
                let msg="unknown type";
                let obj=alloc_string(msg.as_ptr(), msg.len());
                return std::ptr::null_mut() as *mut Object
            },
        }
    };
}}}

#[macro_export]
macro_rules! logic_operation {
    ($left:ident, $binop:tt, $right:ident) => {
        unsafe { {
        use VarType::*;
        const EPS: f64 = 1e-12;
        let left_type = (*$left).ty.clone();
        let right_type = (*$right).ty.clone();

        match (left_type, right_type) {
            (Int, Int) => {
                let left=type_trans!($left, IntObject);
                let right=type_trans!($right, IntObject);

                let left_val=(*left).value;
                let right_val=(*right).value;

                let res=alloc_bool(((left_val != 0) $binop (right_val != 0)));
                return res
            },
            (Int, Float) => {
                let left=type_trans!($left, IntObject);
                let right=type_trans!($right, FloatObject);

                let left_val=(*left).value;
                let right_val=(*right).value;

                let res=alloc_bool((left_val != 0) $binop (right_val.abs() > EPS));
                return res
            },
            (Int, Bool) => {
                let left=type_trans!($left, IntObject);
                let right=type_trans!($right, BoolObject);

                let left_val=(*left).value;
                let right_val=(*right).value;

                let res=alloc_bool((left_val != 0) $binop right_val);
                return res
            },
            (Int, Char) => {
                let left=type_trans!($left, IntObject);
                let right=type_trans!($right, CharObject);

                let left_val=(*left).value;
                let right_val=(*right).value;

                let res=alloc_bool(((left_val != 0) $binop (right_val != 0)));
                return res
            },
            (Int, String) => {
                let left=type_trans!($left, IntObject);
                let right =type_trans!($right, StringObject);

                let left_val=(*left).value;
                let right_len=(*right).len;

                let res=alloc_bool((left_val != 0) $binop (right_len != 0));
                return res
            },
            (Float, Int) => {
                let $left=type_trans!($left, FloatObject);
                let $right=type_trans!($right, IntObject);

                let left_val=(*$left).value;
                let right_val=(*$right).value;

                let res=alloc_bool((left_val.abs() > EPS) $binop (right_val != 0));
                return res
            },
            (Float, Float) => {
                let $left=type_trans!($left, FloatObject);
                let $right=type_trans!($right, FloatObject);

                let left_val=(*$left).value;
                let right_val=(*$right).value;

                let res=alloc_bool((left_val.abs() > EPS) $binop (right_val.abs() > EPS));
                return res
            },
            (Float, Bool) => {
                let $left=type_trans!($left, FloatObject);
                let $right=type_trans!($right, BoolObject);

                let left_val=(*$left).value;
                let right_val=(*$right).value;

                let res=alloc_bool((left_val.abs() > EPS) $binop right_val);
                return res
            },
            (Float, Char) => {
                let left=type_trans!($left, FloatObject);
                let right=type_trans!($right, CharObject);

                let left_val=(*left).value;
                let right_val=(*right).value;

                let res=alloc_bool((left_val.abs() > EPS) $binop (right_val != 0));
                return res
            },
            (Float, String) => {
                let left=type_trans!($left, FloatObject);
                let right =type_trans!($right, StringObject);

                let left_val=(*left).value;
                let right_val=(*right).data;
                let len=(*right).len;

                let left_str=left_val.to_string();

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(len + left_str.len()) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(
                    left_str.as_ptr(), 
                    new_ptr, 
                    left_str.len()
                );
                std::ptr::copy_nonoverlapping(
                    right_val, 
                    new_ptr.add(left_str.len()), 
                    len
                );

                let res=alloc_string(new_ptr, len+left_str.len());
                return res
            },
            (Bool, Int) => {
                let left=type_trans!($left, BoolObject);
                let right=type_trans!($right, IntObject);

                let left_val=(*left).value;
                let right_val=(*right).value;

                let res=alloc_bool(left_val $binop (right_val != 0));
                return res
            },
            (Bool, Float) => {
                let $left=type_trans!($left, BoolObject);
                let $right=type_trans!($right, FloatObject);

                let left_val=(*$left).value;
                let right_val=(*$right).value;

                let res=alloc_bool(left_val $binop (right_val.abs() > EPS));
                return res
            },
            (Bool, Bool) => {
                let $left=type_trans!($left, BoolObject);
                let $right=type_trans!($right, BoolObject);

                let left_val=(*$left).value;
                let right_val=(*$right).value;

                let res=alloc_bool(left_val $binop right_val);
                return res
            },
            (Bool, Char) => {
                let $left=type_trans!($left, BoolObject);
                let $right=type_trans!($right, CharObject);

                let left_val=(*$left).value;
                let right_val=(*$right).value;

                let res=alloc_bool(left_val $binop (right_val != 0));
                return res
            },
            (Bool, String) => {
                let left=type_trans!($left, BoolObject);
                let right =type_trans!($right, StringObject);

                let left_val=(*left).value;
                let len=(*right).len;

                let res=alloc_bool(left_val $binop (len != 0));
                return res
            },
            (Char, Int) => {
                let left=type_trans!($left, CharObject);
                let right=type_trans!($right, IntObject);

                let left_val=(*left).value;
                let right_val=(*right).value;

                let res=alloc_bool(((left_val != 0) $binop (right_val != 0)));
                return res
            },
            (Char, Float) => {
                let left=type_trans!($left, CharObject);
                let right=type_trans!($right, FloatObject);

                let left_val=(*left).value;
                let right_val=(*right).value;

                let res=alloc_bool((left_val != 0) $binop (right_val.abs() >EPS));
                return res
            },
            (Char, Bool) => {
                let $left=type_trans!($left, CharObject);
                let $right=type_trans!($right, BoolObject);

                let left_val=(*$left).value;
                let right_val=(*$right).value;
                let res=alloc_bool((left_val != 0) $binop right_val);
                return res
            },
            (Char, Char) => {
                let $left=type_trans!($left, CharObject);
                let $right=type_trans!($right, CharObject);

                let left_val=(*$left).value;
                let right_val=(*$right).value;

                let res=alloc_bool(((left_val != 0) $binop (right_val != 0)));
                return res
            },
            (Char, String) => {
                let left=type_trans!($left, CharObject);
                let right =type_trans!($right, StringObject);

                let left_val=(*left).value;
                let right_val=(*right).data;
                let len=(*right).len;

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(len + 1) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 2. 写入新字符
                std::ptr::write(new_ptr, left_val);

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(right_val, new_ptr.add(1), len);

                let res=alloc_string(new_ptr, len+1);
                return res
            },
            (String, Int) => {
                let left=type_trans!($left, StringObject);
                let right =type_trans!($right, IntObject);

                let right_val=(*right).value;
                let left_val=(*left).data;
                let len=(*left).len;

                let right_str=right_val.to_string();

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(len + right_str.len()) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(
                    left_val, 
                    new_ptr, 
                    len
                );
                std::ptr::copy_nonoverlapping(
                    right_str.as_ptr(), 
                    new_ptr.add(len), 
                    right_str.len()
                );

                let res=alloc_string(new_ptr, len+right_str.len());
                return res
            },
            (String, Float) => {
                let left=type_trans!($left, StringObject);
                let right =type_trans!($right, FloatObject);

                let right_val=(*right).value;
                let left_val=(*left).data;
                let len=(*left).len;

                let right_str=right_val.to_string();

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(len + right_str.len()) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(
                    left_val, 
                    new_ptr, 
                    len
                );
                std::ptr::copy_nonoverlapping(
                    right_str.as_ptr(), 
                    new_ptr.add(len), 
                    right_str.len()
                );

                let res=alloc_string(new_ptr, len+right_str.len());
                return res
            },
            (String, Bool) => {
                let left=type_trans!($left, StringObject);
                let right =type_trans!($right, BoolObject);

                let right_val=(*right).value;
                let left_val=(*left).data;
                let len=(*left).len;

                let right_str=right_val.to_string();

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(len + right_str.len()) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(
                    left_val, 
                    new_ptr, 
                    len
                );
                std::ptr::copy_nonoverlapping(
                    right_str.as_ptr(), 
                    new_ptr.add(len), 
                    right_str.len()
                );

                let res=alloc_string(new_ptr, len+right_str.len());
                return res
            },
            (String, Char) => {
                let left=type_trans!($left, StringObject);
                let right =type_trans!($right, CharObject);

                let right_val=(*right).value;
                let left_val=(*left).data;
                let len=(*left).len;

                let right_str=right_val.to_string();

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(len + right_str.len()) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(
                    left_val, 
                    new_ptr, 
                    len
                );
                std::ptr::copy_nonoverlapping(
                    right_str.as_ptr(), 
                    new_ptr.add(len), 
                    right_str.len()
                );

                let res=alloc_string(new_ptr, len+right_str.len());
                return res
            },
            (String, String) => {
                let left=type_trans!($left, StringObject);
                let right =type_trans!($right, StringObject);

                let left_val=(*left).data;
                let left_len=(*left).len;
                let right_val=(*right).data;
                let right_len=(*right).len;

                // 1. 重新分配一块更大的内存
                let new_ptr = libc::malloc(left_len + right_len) as *mut u8;
                if new_ptr.is_null() {
                    panic!("alloc failed");
                }

                // 3. 拷贝原内容
                std::ptr::copy_nonoverlapping(
                    left_val, 
                    new_ptr, 
                    left_len
                );
                std::ptr::copy_nonoverlapping(
                    right_val, 
                    new_ptr.add(left_len), 
                    right_len
                );

                let res=alloc_string(new_ptr, left_len+right_len);
                return res
            },
            (Unknown, _) => {
                let msg="unknown type";
                let obj=alloc_string(msg.as_ptr(), msg.len());
                return std::ptr::null_mut() as *mut Object
            },
            (_,Unknown) => {
                let msg="unknown type";
                let obj=alloc_string(msg.as_ptr(), msg.len());
                return std::ptr::null_mut() as *mut Object
            },
        }
    };
}}}