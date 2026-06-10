; ModuleID = 'ayanami'
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

%struct.String = type { ptr, i64 }

define ptr @to_string_unique_float_vtable_float_ToString_wrap(ptr %data) {
  %val = load double, ptr %data, align 8
  %e0 = call ptr @to_string_unique_float(double %val)
  ret ptr %e0
}

define ptr @to_string_unique_char_vtable_char_ToString_wrap(ptr %data) {
  %val = load i8, ptr %data, align 8
  %e1 = call ptr @to_string_unique_char(i8 %val)
  ret ptr %e1
}

define ptr @to_string_unique_int_vtable_int_ToString_wrap(ptr %data) {
  %val = load i64, ptr %data, align 8
  %e2 = call ptr @to_string_unique_int(i64 %val)
  ret ptr %e2
}

define ptr @to_string_unique_bool_vtable_bool_ToString_wrap(ptr %data) {
  %val = load i1, ptr %data, align 8
  %e3 = call ptr @to_string_unique_bool(i1 %val)
  ret ptr %e3
}

@vtable_float_ToString = private unnamed_addr constant [2 x ptr] [
  ptr null,
  ptr @to_string_unique_float_vtable_float_ToString_wrap
]

@vtable_String_ToString = private unnamed_addr constant [2 x ptr] [
  ptr null,
  ptr @to_string_unique_String
]

@vtable_char_ToString = private unnamed_addr constant [2 x ptr] [
  ptr null,
  ptr @to_string_unique_char_vtable_char_ToString_wrap
]

@vtable_int_ToString = private unnamed_addr constant [2 x ptr] [
  ptr null,
  ptr @to_string_unique_int_vtable_int_ToString_wrap
]

@vtable_bool_ToString = private unnamed_addr constant [2 x ptr] [
  ptr null,
  ptr @to_string_unique_bool_vtable_bool_ToString_wrap
]

declare void @free(i8*)
declare i8* @malloc(i64)
declare i8* @__ayanami_shared_alloc(i64)
declare void @__ayanami_shared_retain(i8*)
declare void @__ayanami_shared_release(i8*)
declare void @llvm.memcpy.p0.p0.i64(i8*, i8*, i64, i1)
declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)
declare i32 @putchar(i32)
declare i32 @printf(i8*, ...)

declare i64 @__ayanami_float_str()
declare i64 @__ayanami_float_len()

define ptr @to_string_unique_int(i64) {
  %v0 = alloca i64, align 8
  %v1 = alloca i64, align 8
  %v2 = alloca i64, align 8
  %v3 = alloca i64, align 8
  %v4 = alloca ptr, align 8
  %v5 = alloca ptr, align 8
  %v6 = alloca i64, align 8
  store i64 %0, ptr %v0, align 8
  %t0 = load i64, ptr %v0, align 8
  %t1 = icmp eq i64 %t0, 0
  br i1 %t1, label %then0, label %else1
  then0:
  %t3 = call i8* @malloc(i64 1)
  %t2 = bitcast i8* %t3 to ptr
  %t4 = getelementptr i8, ptr %t2, i64 0
  store i8 48, ptr %t4
  %t6 = alloca %struct.String, align 8
  %t7 = getelementptr %struct.String, ptr %t6, i32 0, i32 0
  store ptr %t2, ptr %t7
  %t8 = getelementptr %struct.String, ptr %t6, i32 0, i32 1
  store i64 1, ptr %t8
  %t5 = load %struct.String, ptr %t6
  %t10 = alloca %struct.String, align 8
  store %struct.String %t5, ptr %t10
  %l11 = call i8* @malloc(i64 16)
  call void @llvm.memcpy.p0.p0.i64(i8* %l11, ptr %t10, i64 16, i1 false)
  %t9 = bitcast i8* %l11 to ptr
  ret ptr %t9
  else1:
  br label %ifcont2
  ifcont2:
  %t12 = load i64, ptr %v0, align 8
  store i64 %t12, ptr %v1, align 8
  store i64 0, ptr %v2, align 8
  %t13 = load i64, ptr %v1, align 8
  %t14 = icmp slt i64 %t13, 0
  br i1 %t14, label %then3, label %else4
  then3:
  store i64 1, ptr %v2, align 8
  %t15 = load i64, ptr %v1, align 8
  %t16 = call i64 @neg_unique_int(i64 %t15)
  store i64 %t16, ptr %v1, align 8
  br label %ifcont5
  else4:
  br label %ifcont5
  ifcont5:
  %t17 = load i64, ptr %v1, align 8
  store i64 %t17, ptr %v3, align 8
  br label %while.cond6
  while.cond6:
  %t18 = load i64, ptr %v3, align 8
  %t19 = icmp sgt i64 %t18, 0
  br i1 %t19, label %while.body7, label %while.end8
  while.body7:
  %t20 = load i64, ptr %v2, align 8
  %t21 = add i64 %t20, 1
  store i64 %t21, ptr %v2, align 8
  %t22 = load i64, ptr %v3, align 8
  %t23 = sdiv i64 %t22, 10
  store i64 %t23, ptr %v3, align 8
  br label %while.cond6
  while.end8:
  %t25 = call i8* @malloc(i64 10)
  %t24 = bitcast i8* %t25 to ptr
  %t26 = getelementptr i8, ptr %t24, i64 0
  store i8 48, ptr %t26
  %t27 = getelementptr i8, ptr %t24, i64 1
  store i8 49, ptr %t27
  %t28 = getelementptr i8, ptr %t24, i64 2
  store i8 50, ptr %t28
  %t29 = getelementptr i8, ptr %t24, i64 3
  store i8 51, ptr %t29
  %t30 = getelementptr i8, ptr %t24, i64 4
  store i8 52, ptr %t30
  %t31 = getelementptr i8, ptr %t24, i64 5
  store i8 53, ptr %t31
  %t32 = getelementptr i8, ptr %t24, i64 6
  store i8 54, ptr %t32
  %t33 = getelementptr i8, ptr %t24, i64 7
  store i8 55, ptr %t33
  %t34 = getelementptr i8, ptr %t24, i64 8
  store i8 56, ptr %t34
  %t35 = getelementptr i8, ptr %t24, i64 9
  store i8 57, ptr %t35
  store ptr %t24, ptr %v4, align 8
  %t40 = load i64, ptr %v2, align 8
  %t38 = add i64 0, %t40
  %t39 = mul i64 %t38, 1
  %t37 = call i8* @malloc(i64 %t39)
  %t36 = bitcast i8* %t37 to ptr
  call void @llvm.memset.p0.i64(ptr %t36, i8 0, i64 %t39, i1 false)
  store ptr %t36, ptr %v5, align 8
  %t41 = load i64, ptr %v2, align 8
  %t42 = sub i64 %t41, 1
  store i64 %t42, ptr %v6, align 8
  %t43 = load i64, ptr %v0, align 8
  %t44 = icmp slt i64 %t43, 0
  br i1 %t44, label %then9, label %else10
  then9:
  %t45 = load ptr, ptr %v5, align 8
  %t46 = getelementptr i8, ptr %t45, i64 0
  store i8 45, ptr %t46
  %t47 = load i64, ptr %v0, align 8
  %t48 = call i64 @neg_unique_int(i64 %t47)
  store i64 %t48, ptr %v1, align 8
  br label %ifcont11
  else10:
  %t49 = load i64, ptr %v0, align 8
  store i64 %t49, ptr %v1, align 8
  br label %ifcont11
  ifcont11:
  br label %while.cond12
  while.cond12:
  %t50 = load i64, ptr %v1, align 8
  %t51 = icmp sgt i64 %t50, 0
  br i1 %t51, label %while.body13, label %while.end14
  while.body13:
  %t52 = load ptr, ptr %v5, align 8
  %t53 = load i64, ptr %v6, align 8
  %t54 = load ptr, ptr %v4, align 8
  %t55 = load i64, ptr %v1, align 8
  %t56 = srem i64 %t55, 10
  %t58 = getelementptr i8, ptr %t54, i64 %t56
  %t59 = load i8, ptr %t58
  %t57 = bitcast i8 %t59 to i8
  %t60 = getelementptr i8, ptr %t52, i64 %t53
  store i8 %t57, ptr %t60
  %t61 = load i64, ptr %v1, align 8
  %t62 = sdiv i64 %t61, 10
  store i64 %t62, ptr %v1, align 8
  %t63 = load i64, ptr %v6, align 8
  %t64 = sub i64 %t63, 1
  store i64 %t64, ptr %v6, align 8
  br label %while.cond12
  while.end14:
  %t65 = load ptr, ptr %v5, align 8
  %t66 = load i64, ptr %v2, align 8
  %t68 = alloca %struct.String, align 8
  %t69 = getelementptr %struct.String, ptr %t68, i32 0, i32 0
  store ptr %t65, ptr %t69
  %t70 = getelementptr %struct.String, ptr %t68, i32 0, i32 1
  store i64 %t66, ptr %t70
  %t67 = load %struct.String, ptr %t68
  %t72 = alloca %struct.String, align 8
  store %struct.String %t67, ptr %t72
  %l73 = call i8* @malloc(i64 16)
  call void @llvm.memcpy.p0.p0.i64(i8* %l73, ptr %t72, i64 16, i1 false)
  %t71 = bitcast i8* %l73 to ptr
  ret ptr %t71
}

define ptr @to_string_unique_float(double) {
  %v0 = alloca double, align 8
  %v1 = alloca i64, align 8
  %v2 = alloca ptr, align 8
  store double %0, ptr %v0, align 8
  %t0 = load double, ptr %v0, align 8
  %t1 = call i64 @__ayanami_float_len(double %t0)
  store i64 %t1, ptr %v1, align 8
  %t2 = load double, ptr %v0, align 8
  %t3 = load i64, ptr %v1, align 8
  %t4 = call ptr @__ayanami_float_str(double %t2, i64 %t3)
  store ptr %t4, ptr %v2, align 8
  %t5 = load ptr, ptr %v2, align 8
  %t6 = load i64, ptr %v1, align 8
  %t8 = alloca %struct.String, align 8
  %t9 = getelementptr %struct.String, ptr %t8, i32 0, i32 0
  store ptr %t5, ptr %t9
  %t10 = getelementptr %struct.String, ptr %t8, i32 0, i32 1
  store i64 %t6, ptr %t10
  %t7 = load %struct.String, ptr %t8
  %t12 = alloca %struct.String, align 8
  store %struct.String %t7, ptr %t12
  %l13 = call i8* @malloc(i64 16)
  call void @llvm.memcpy.p0.p0.i64(i8* %l13, ptr %t12, i64 16, i1 false)
  %t11 = bitcast i8* %l13 to ptr
  ret ptr %t11
}

define ptr @to_string_unique_char(i8) {
  %v0 = alloca i8, align 8
  store i8 %0, ptr %v0, align 8
  %t0 = load i8, ptr %v0, align 8
  %t2 = call i8* @malloc(i64 1)
  %t1 = bitcast i8* %t2 to ptr
  %t3 = getelementptr i8, ptr %t1, i64 0
  store i8 %t0, ptr %t3
  %t5 = alloca %struct.String, align 8
  %t6 = getelementptr %struct.String, ptr %t5, i32 0, i32 0
  store ptr %t1, ptr %t6
  %t7 = getelementptr %struct.String, ptr %t5, i32 0, i32 1
  store i64 1, ptr %t7
  %t4 = load %struct.String, ptr %t5
  %t9 = alloca %struct.String, align 8
  store %struct.String %t4, ptr %t9
  %l10 = call i8* @malloc(i64 16)
  call void @llvm.memcpy.p0.p0.i64(i8* %l10, ptr %t9, i64 16, i1 false)
  %t8 = bitcast i8* %l10 to ptr
  ret ptr %t8
}

define ptr @to_string_unique_bool(i1) {
  %v0 = alloca i1, align 8
  store i1 %0, ptr %v0, align 8
  %t0 = load i1, ptr %v0, align 8
  br i1 %t0, label %then0, label %else1
  then0:
  %t2 = call i8* @malloc(i64 4)
  %t1 = bitcast i8* %t2 to ptr
  %t3 = getelementptr i8, ptr %t1, i64 0
  store i8 116, ptr %t3
  %t4 = getelementptr i8, ptr %t1, i64 1
  store i8 114, ptr %t4
  %t5 = getelementptr i8, ptr %t1, i64 2
  store i8 117, ptr %t5
  %t6 = getelementptr i8, ptr %t1, i64 3
  store i8 101, ptr %t6
  %t8 = alloca %struct.String, align 8
  %t9 = getelementptr %struct.String, ptr %t8, i32 0, i32 0
  store ptr %t1, ptr %t9
  %t10 = getelementptr %struct.String, ptr %t8, i32 0, i32 1
  store i64 4, ptr %t10
  %t7 = load %struct.String, ptr %t8
  %t12 = alloca %struct.String, align 8
  store %struct.String %t7, ptr %t12
  %l13 = call i8* @malloc(i64 16)
  call void @llvm.memcpy.p0.p0.i64(i8* %l13, ptr %t12, i64 16, i1 false)
  %t11 = bitcast i8* %l13 to ptr
  ret ptr %t11
  else1:
  %t15 = call i8* @malloc(i64 5)
  %t14 = bitcast i8* %t15 to ptr
  %t16 = getelementptr i8, ptr %t14, i64 0
  store i8 102, ptr %t16
  %t17 = getelementptr i8, ptr %t14, i64 1
  store i8 97, ptr %t17
  %t18 = getelementptr i8, ptr %t14, i64 2
  store i8 108, ptr %t18
  %t19 = getelementptr i8, ptr %t14, i64 3
  store i8 115, ptr %t19
  %t20 = getelementptr i8, ptr %t14, i64 4
  store i8 101, ptr %t20
  %t22 = alloca %struct.String, align 8
  %t23 = getelementptr %struct.String, ptr %t22, i32 0, i32 0
  store ptr %t14, ptr %t23
  %t24 = getelementptr %struct.String, ptr %t22, i32 0, i32 1
  store i64 5, ptr %t24
  %t21 = load %struct.String, ptr %t22
  %t26 = alloca %struct.String, align 8
  store %struct.String %t21, ptr %t26
  %l27 = call i8* @malloc(i64 16)
  call void @llvm.memcpy.p0.p0.i64(i8* %l27, ptr %t26, i64 16, i1 false)
  %t25 = bitcast i8* %l27 to ptr
  ret ptr %t25
  br label %ifcont2
  ifcont2:
  ret ptr null
}

define i64 @add_unique_int_int(i64, i64) {
  %v0 = alloca i64, align 8
  %v1 = alloca i64, align 8
  store i64 %0, ptr %v0, align 8
  store i64 %1, ptr %v1, align 8
  %t0 = load i64, ptr %v0, align 8
  %t1 = load i64, ptr %v1, align 8
  %t2 = add i64 %t0, %t1
  ret i64 %t2
}

define i64 @sub_unique_int_int(i64, i64) {
  %v0 = alloca i64, align 8
  %v1 = alloca i64, align 8
  store i64 %0, ptr %v0, align 8
  store i64 %1, ptr %v1, align 8
  %t0 = load i64, ptr %v0, align 8
  %t1 = load i64, ptr %v1, align 8
  %t2 = sub i64 %t0, %t1
  ret i64 %t2
}

define i64 @mul_unique_int_int(i64, i64) {
  %v0 = alloca i64, align 8
  %v1 = alloca i64, align 8
  store i64 %0, ptr %v0, align 8
  store i64 %1, ptr %v1, align 8
  %t0 = load i64, ptr %v0, align 8
  %t1 = load i64, ptr %v1, align 8
  %t2 = mul i64 %t0, %t1
  ret i64 %t2
}

define i64 @div_unique_int_int(i64, i64) {
  %v0 = alloca i64, align 8
  %v1 = alloca i64, align 8
  store i64 %0, ptr %v0, align 8
  store i64 %1, ptr %v1, align 8
  %t0 = load i64, ptr %v0, align 8
  %t1 = load i64, ptr %v1, align 8
  %t2 = sdiv i64 %t0, %t1
  ret i64 %t2
}

define i64 @rem_unique_int_int(i64, i64) {
  %v0 = alloca i64, align 8
  %v1 = alloca i64, align 8
  store i64 %0, ptr %v0, align 8
  store i64 %1, ptr %v1, align 8
  %t0 = load i64, ptr %v0, align 8
  %t1 = load i64, ptr %v1, align 8
  %t2 = srem i64 %t0, %t1
  ret i64 %t2
}

define i1 @eq_unique_int_int(i64, i64) {
  %v0 = alloca i64, align 8
  %v1 = alloca i64, align 8
  store i64 %0, ptr %v0, align 8
  store i64 %1, ptr %v1, align 8
  %t0 = load i64, ptr %v0, align 8
  %t1 = load i64, ptr %v1, align 8
  %t2 = icmp eq i64 %t0, %t1
  ret i64 %t2
}

define i1 @ne_unique_int_int(i64, i64) {
  %v0 = alloca i64, align 8
  %v1 = alloca i64, align 8
  store i64 %0, ptr %v0, align 8
  store i64 %1, ptr %v1, align 8
  %t0 = load i64, ptr %v0, align 8
  %t1 = load i64, ptr %v1, align 8
  %t2 = icmp ne i64 %t0, %t1
  ret i64 %t2
}

define i1 @lt_unique_int_int(i64, i64) {
  %v0 = alloca i64, align 8
  %v1 = alloca i64, align 8
  store i64 %0, ptr %v0, align 8
  store i64 %1, ptr %v1, align 8
  %t0 = load i64, ptr %v0, align 8
  %t1 = load i64, ptr %v1, align 8
  %t2 = icmp slt i64 %t0, %t1
  ret i64 %t2
}

define i1 @gt_unique_int_int(i64, i64) {
  %v0 = alloca i64, align 8
  %v1 = alloca i64, align 8
  store i64 %0, ptr %v0, align 8
  store i64 %1, ptr %v1, align 8
  %t0 = load i64, ptr %v0, align 8
  %t1 = load i64, ptr %v1, align 8
  %t2 = icmp sgt i64 %t0, %t1
  ret i64 %t2
}

define i1 @le_unique_int_int(i64, i64) {
  %v0 = alloca i64, align 8
  %v1 = alloca i64, align 8
  store i64 %0, ptr %v0, align 8
  store i64 %1, ptr %v1, align 8
  %t0 = load i64, ptr %v0, align 8
  %t1 = load i64, ptr %v1, align 8
  %t2 = icmp sle i64 %t0, %t1
  ret i64 %t2
}

define i1 @ge_unique_int_int(i64, i64) {
  %v0 = alloca i64, align 8
  %v1 = alloca i64, align 8
  store i64 %0, ptr %v0, align 8
  store i64 %1, ptr %v1, align 8
  %t0 = load i64, ptr %v0, align 8
  %t1 = load i64, ptr %v1, align 8
  %t2 = icmp sge i64 %t0, %t1
  ret i64 %t2
}

define i64 @neg_unique_int(i64) {
  %v0 = alloca i64, align 8
  store i64 %0, ptr %v0, align 8
  %t0 = load i64, ptr %v0, align 8
  %t1 = call i64 @neg_unique_int(i64 %t0)
  ret i64 %t1
}

define double @add_unique_float_float(double, double) {
  %v0 = alloca double, align 8
  %v1 = alloca double, align 8
  store double %0, ptr %v0, align 8
  store double %1, ptr %v1, align 8
  %t0 = load double, ptr %v0, align 8
  %t1 = load double, ptr %v1, align 8
  %t2 = fadd double %t0, %t1
  ret double %t2
}

define double @sub_unique_float_float(double, double) {
  %v0 = alloca double, align 8
  %v1 = alloca double, align 8
  store double %0, ptr %v0, align 8
  store double %1, ptr %v1, align 8
  %t0 = load double, ptr %v0, align 8
  %t1 = load double, ptr %v1, align 8
  %t2 = fsub double %t0, %t1
  ret double %t2
}

define double @mul_unique_float_float(double, double) {
  %v0 = alloca double, align 8
  %v1 = alloca double, align 8
  store double %0, ptr %v0, align 8
  store double %1, ptr %v1, align 8
  %t0 = load double, ptr %v0, align 8
  %t1 = load double, ptr %v1, align 8
  %t2 = fmul double %t0, %t1
  ret double %t2
}

define double @div_unique_float_float(double, double) {
  %v0 = alloca double, align 8
  %v1 = alloca double, align 8
  store double %0, ptr %v0, align 8
  store double %1, ptr %v1, align 8
  %t0 = load double, ptr %v0, align 8
  %t1 = load double, ptr %v1, align 8
  %t2 = fdiv double %t0, %t1
  ret double %t2
}

define i1 @eq_unique_float_float(double, double) {
  %v0 = alloca double, align 8
  %v1 = alloca double, align 8
  store double %0, ptr %v0, align 8
  store double %1, ptr %v1, align 8
  %t0 = load double, ptr %v0, align 8
  %t1 = load double, ptr %v1, align 8
  %t2 = fcmp oeq double %t0, %t1
  ret double %t2
}

define i1 @ne_unique_float_float(double, double) {
  %v0 = alloca double, align 8
  %v1 = alloca double, align 8
  store double %0, ptr %v0, align 8
  store double %1, ptr %v1, align 8
  %t0 = load double, ptr %v0, align 8
  %t1 = load double, ptr %v1, align 8
  %t2 = fcmp one double %t0, %t1
  ret double %t2
}

define i1 @lt_unique_float_float(double, double) {
  %v0 = alloca double, align 8
  %v1 = alloca double, align 8
  store double %0, ptr %v0, align 8
  store double %1, ptr %v1, align 8
  %t0 = load double, ptr %v0, align 8
  %t1 = load double, ptr %v1, align 8
  %t2 = fcmp olt double %t0, %t1
  ret double %t2
}

define i1 @gt_unique_float_float(double, double) {
  %v0 = alloca double, align 8
  %v1 = alloca double, align 8
  store double %0, ptr %v0, align 8
  store double %1, ptr %v1, align 8
  %t0 = load double, ptr %v0, align 8
  %t1 = load double, ptr %v1, align 8
  %t2 = fcmp ogt double %t0, %t1
  ret double %t2
}

define i1 @le_unique_float_float(double, double) {
  %v0 = alloca double, align 8
  %v1 = alloca double, align 8
  store double %0, ptr %v0, align 8
  store double %1, ptr %v1, align 8
  %t0 = load double, ptr %v0, align 8
  %t1 = load double, ptr %v1, align 8
  %t2 = fcmp ole double %t0, %t1
  ret double %t2
}

define i1 @ge_unique_float_float(double, double) {
  %v0 = alloca double, align 8
  %v1 = alloca double, align 8
  store double %0, ptr %v0, align 8
  store double %1, ptr %v1, align 8
  %t0 = load double, ptr %v0, align 8
  %t1 = load double, ptr %v1, align 8
  %t2 = fcmp oge double %t0, %t1
  ret double %t2
}

define double @neg_unique_float(double) {
  %v0 = alloca double, align 8
  store double %0, ptr %v0, align 8
  %t0 = load double, ptr %v0, align 8
  %t1 = call double @neg_unique_float(double %t0)
  ret double %t1
}

define i1 @eq_unique_char_char(i8, i8) {
  %v0 = alloca i8, align 8
  %v1 = alloca i8, align 8
  store i8 %0, ptr %v0, align 8
  store i8 %1, ptr %v1, align 8
  %t0 = load i8, ptr %v0, align 8
  %t1 = load i8, ptr %v1, align 8
  %t2 = icmp eq i8 %t0, %t1
  ret i8 %t2
}

define i1 @ne_unique_char_char(i8, i8) {
  %v0 = alloca i8, align 8
  %v1 = alloca i8, align 8
  store i8 %0, ptr %v0, align 8
  store i8 %1, ptr %v1, align 8
  %t0 = load i8, ptr %v0, align 8
  %t1 = load i8, ptr %v1, align 8
  %t2 = icmp ne i8 %t0, %t1
  ret i8 %t2
}

define i1 @lt_unique_char_char(i8, i8) {
  %v0 = alloca i8, align 8
  %v1 = alloca i8, align 8
  store i8 %0, ptr %v0, align 8
  store i8 %1, ptr %v1, align 8
  %t0 = load i8, ptr %v0, align 8
  %t1 = load i8, ptr %v1, align 8
  %t2 = icmp slt i8 %t0, %t1
  ret i8 %t2
}

define i1 @gt_unique_char_char(i8, i8) {
  %v0 = alloca i8, align 8
  %v1 = alloca i8, align 8
  store i8 %0, ptr %v0, align 8
  store i8 %1, ptr %v1, align 8
  %t0 = load i8, ptr %v0, align 8
  %t1 = load i8, ptr %v1, align 8
  %t2 = icmp sgt i8 %t0, %t1
  ret i8 %t2
}

define i1 @le_unique_char_char(i8, i8) {
  %v0 = alloca i8, align 8
  %v1 = alloca i8, align 8
  store i8 %0, ptr %v0, align 8
  store i8 %1, ptr %v1, align 8
  %t0 = load i8, ptr %v0, align 8
  %t1 = load i8, ptr %v1, align 8
  %t2 = icmp sle i8 %t0, %t1
  ret i8 %t2
}

define i1 @ge_unique_char_char(i8, i8) {
  %v0 = alloca i8, align 8
  %v1 = alloca i8, align 8
  store i8 %0, ptr %v0, align 8
  store i8 %1, ptr %v1, align 8
  %t0 = load i8, ptr %v0, align 8
  %t1 = load i8, ptr %v1, align 8
  %t2 = icmp sge i8 %t0, %t1
  ret i8 %t2
}

define i1 @eq_unique_bool_bool(i1, i1) {
  %v0 = alloca i1, align 8
  %v1 = alloca i1, align 8
  store i1 %0, ptr %v0, align 8
  store i1 %1, ptr %v1, align 8
  %t0 = load i1, ptr %v0, align 8
  %t1 = load i1, ptr %v1, align 8
  %t2 = icmp eq i1 %t0, %t1
  ret i1 %t2
}

define i1 @ne_unique_bool_bool(i1, i1) {
  %v0 = alloca i1, align 8
  %v1 = alloca i1, align 8
  store i1 %0, ptr %v0, align 8
  store i1 %1, ptr %v1, align 8
  %t0 = load i1, ptr %v0, align 8
  %t1 = load i1, ptr %v1, align 8
  %t2 = icmp ne i1 %t0, %t1
  ret i1 %t2
}

define ptr @to_string_unique_String(ptr) {
  %v0 = alloca ptr, align 8
  store ptr %0, ptr %v0, align 8
  %t0 = load ptr, ptr %v0, align 8
  %t1 = call ptr @copy_unique_String(ptr %t0)
  ret ptr %t1
}

define i8 @index_unique_String_int(ptr, i64) {
  %v0 = alloca ptr, align 8
  %v1 = alloca i64, align 8
  store ptr %0, ptr %v0, align 8
  store i64 %1, ptr %v1, align 8
  %t0 = load ptr, ptr %v0, align 8
  %t2 = getelementptr %struct.String, ptr %t0, i32 0, i32 0
  %t1 = load ptr, ptr %t2
  %t3 = load i64, ptr %v1, align 8
  %t5 = getelementptr i8, ptr %t1, i64 %t3
  %t6 = load i8, ptr %t5
  %t4 = bitcast i8 %t6 to i8
  ret i8 %t4
}

define i64 @len_unique_String(ptr) {
  %v0 = alloca ptr, align 8
  store ptr %0, ptr %v0, align 8
  %t0 = load ptr, ptr %v0, align 8
  %t2 = getelementptr %struct.String, ptr %t0, i32 0, i32 1
  %t1 = load i64, ptr %t2
  ret i64 %t1
}

define ptr @add_unique_String_unique_String(ptr, ptr) {
  %v0 = alloca ptr, align 8
  %v1 = alloca ptr, align 8
  %v2 = alloca i64, align 8
  %v3 = alloca ptr, align 8
  %v4 = alloca i64, align 8
  %v5 = alloca i64, align 8
  store ptr %0, ptr %v0, align 8
  store ptr %1, ptr %v1, align 8
  %t0 = load ptr, ptr %v0, align 8
  %t2 = getelementptr %struct.String, ptr %t0, i32 0, i32 1
  %t1 = load i64, ptr %t2
  %t3 = load ptr, ptr %v1, align 8
  %t5 = getelementptr %struct.String, ptr %t3, i32 0, i32 1
  %t4 = load i64, ptr %t5
  %t6 = add i64 %t1, %t4
  store i64 %t6, ptr %v2, align 8
  %t11 = load i64, ptr %v2, align 8
  %t9 = add i64 0, %t11
  %t10 = mul i64 %t9, 1
  %t8 = call i8* @malloc(i64 %t10)
  %t7 = bitcast i8* %t8 to ptr
  call void @llvm.memset.p0.i64(ptr %t7, i8 0, i64 %t10, i1 false)
  store ptr %t7, ptr %v3, align 8
  store i64 0, ptr %v4, align 8
  br label %while.cond0
  while.cond0:
  %t12 = load i64, ptr %v4, align 8
  %t13 = load ptr, ptr %v0, align 8
  %t15 = getelementptr %struct.String, ptr %t13, i32 0, i32 1
  %t14 = load i64, ptr %t15
  %t16 = icmp slt i64 %t12, %t14
  br i1 %t16, label %while.body1, label %while.end2
  while.body1:
  %t17 = load ptr, ptr %v3, align 8
  %t18 = load i64, ptr %v4, align 8
  %t19 = load ptr, ptr %v0, align 8
  %t21 = getelementptr %struct.String, ptr %t19, i32 0, i32 0
  %t20 = load ptr, ptr %t21
  %t22 = load i64, ptr %v4, align 8
  %t24 = getelementptr i8, ptr %t20, i64 %t22
  %t25 = load i8, ptr %t24
  %t23 = bitcast i8 %t25 to i8
  %t26 = getelementptr i8, ptr %t17, i64 %t18
  store i8 %t23, ptr %t26
  %t27 = load i64, ptr %v4, align 8
  %t28 = add i64 %t27, 1
  store i64 %t28, ptr %v4, align 8
  br label %while.cond0
  while.end2:
  store i64 0, ptr %v5, align 8
  br label %while.cond3
  while.cond3:
  %t29 = load i64, ptr %v5, align 8
  %t30 = load ptr, ptr %v1, align 8
  %t32 = getelementptr %struct.String, ptr %t30, i32 0, i32 1
  %t31 = load i64, ptr %t32
  %t33 = icmp slt i64 %t29, %t31
  br i1 %t33, label %while.body4, label %while.end5
  while.body4:
  %t34 = load ptr, ptr %v3, align 8
  %t35 = load i64, ptr %v5, align 8
  %t36 = load ptr, ptr %v0, align 8
  %t38 = getelementptr %struct.String, ptr %t36, i32 0, i32 1
  %t37 = load i64, ptr %t38
  %t39 = add i64 %t35, %t37
  %t40 = load ptr, ptr %v1, align 8
  %t42 = getelementptr %struct.String, ptr %t40, i32 0, i32 0
  %t41 = load ptr, ptr %t42
  %t43 = load i64, ptr %v5, align 8
  %t45 = getelementptr i8, ptr %t41, i64 %t43
  %t46 = load i8, ptr %t45
  %t44 = bitcast i8 %t46 to i8
  %t47 = getelementptr i8, ptr %t34, i64 %t39
  store i8 %t44, ptr %t47
  %t48 = load i64, ptr %v5, align 8
  %t49 = add i64 %t48, 1
  store i64 %t49, ptr %v5, align 8
  br label %while.cond3
  while.end5:
  %c4 = load ptr, ptr %v0, align 8
  call void @free(i8* %c4)
  %c5 = load ptr, ptr %v1, align 8
  call void @free(i8* %c5)
  %t50 = load ptr, ptr %v3, align 8
  %t51 = load i64, ptr %v2, align 8
  %t53 = alloca %struct.String, align 8
  %t54 = getelementptr %struct.String, ptr %t53, i32 0, i32 0
  store ptr %t50, ptr %t54
  %t55 = getelementptr %struct.String, ptr %t53, i32 0, i32 1
  store i64 %t51, ptr %t55
  %t52 = load %struct.String, ptr %t53
  %t57 = alloca %struct.String, align 8
  store %struct.String %t52, ptr %t57
  %l58 = call i8* @malloc(i64 16)
  call void @llvm.memcpy.p0.p0.i64(i8* %l58, ptr %t57, i64 16, i1 false)
  %t56 = bitcast i8* %l58 to ptr
  ret ptr %t56
}

define i1 @eq_unique_String_unique_String(ptr, ptr) {
  %v0 = alloca ptr, align 8
  %v1 = alloca ptr, align 8
  %v2 = alloca i64, align 8
  store ptr %0, ptr %v0, align 8
  store ptr %1, ptr %v1, align 8
  %t0 = load ptr, ptr %v0, align 8
  %t2 = getelementptr %struct.String, ptr %t0, i32 0, i32 1
  %t1 = load i64, ptr %t2
  %t3 = load ptr, ptr %v1, align 8
  %t5 = getelementptr %struct.String, ptr %t3, i32 0, i32 1
  %t4 = load i64, ptr %t5
  %t6 = icmp ne i64 %t1, %t4
  br i1 %t6, label %then0, label %else1
  then0:
  %c6 = load ptr, ptr %v0, align 8
  call void @free(i8* %c6)
  %c7 = load ptr, ptr %v1, align 8
  call void @free(i8* %c7)
  ret i1 0
  else1:
  br label %ifcont2
  ifcont2:
  store i64 0, ptr %v2, align 8
  br label %while.cond3
  while.cond3:
  %t7 = load i64, ptr %v2, align 8
  %t8 = load ptr, ptr %v0, align 8
  %t10 = getelementptr %struct.String, ptr %t8, i32 0, i32 1
  %t9 = load i64, ptr %t10
  %t11 = icmp slt i64 %t7, %t9
  br i1 %t11, label %while.body4, label %while.end5
  while.body4:
  %t12 = load ptr, ptr %v0, align 8
  %t14 = getelementptr %struct.String, ptr %t12, i32 0, i32 0
  %t13 = load ptr, ptr %t14
  %t15 = load i64, ptr %v2, align 8
  %t17 = getelementptr i8, ptr %t13, i64 %t15
  %t18 = load i8, ptr %t17
  %t16 = bitcast i8 %t18 to i8
  %t19 = load ptr, ptr %v1, align 8
  %t21 = getelementptr %struct.String, ptr %t19, i32 0, i32 0
  %t20 = load ptr, ptr %t21
  %t22 = load i64, ptr %v2, align 8
  %t24 = getelementptr i8, ptr %t20, i64 %t22
  %t25 = load i8, ptr %t24
  %t23 = bitcast i8 %t25 to i8
  %t26 = icmp ne i8 %t16, %t23
  br i1 %t26, label %then6, label %else7
  then6:
  ret i1 0
  else7:
  br label %ifcont8
  ifcont8:
  %t27 = load i64, ptr %v2, align 8
  %t28 = add i64 %t27, 1
  store i64 %t28, ptr %v2, align 8
  br label %while.cond3
  while.end5:
  ret i1 1
}

define i1 @ne_unique_String_unique_String(ptr, ptr) {
  %v0 = alloca ptr, align 8
  %v1 = alloca ptr, align 8
  store ptr %0, ptr %v0, align 8
  store ptr %1, ptr %v1, align 8
  %t0 = load ptr, ptr %v0, align 8
  %t1 = load ptr, ptr %v1, align 8
  %t2 = call i1 @eq_unique_String_unique_String(ptr %t0, ptr %t1)
  %t3 = xor i1 1, %t2
  ret i1 %t3
}

define ptr @copy_unique_String(ptr) {
  %v0 = alloca ptr, align 8
  %v1 = alloca ptr, align 8
  %v2 = alloca i64, align 8
  store ptr %0, ptr %v0, align 8
  %t4 = load ptr, ptr %v0, align 8
  %t6 = getelementptr %struct.String, ptr %t4, i32 0, i32 1
  %t5 = load i64, ptr %t6
  %t2 = add i64 0, %t5
  %t3 = mul i64 %t2, 1
  %t1 = call i8* @malloc(i64 %t3)
  %t0 = bitcast i8* %t1 to ptr
  call void @llvm.memset.p0.i64(ptr %t0, i8 0, i64 %t3, i1 false)
  store ptr %t0, ptr %v1, align 8
  store i64 0, ptr %v2, align 8
  br label %while.cond0
  while.cond0:
  %t7 = load i64, ptr %v2, align 8
  %t8 = load ptr, ptr %v0, align 8
  %t10 = getelementptr %struct.String, ptr %t8, i32 0, i32 1
  %t9 = load i64, ptr %t10
  %t11 = icmp slt i64 %t7, %t9
  br i1 %t11, label %while.body1, label %while.end2
  while.body1:
  %t12 = load ptr, ptr %v1, align 8
  %t13 = load i64, ptr %v2, align 8
  %t14 = load ptr, ptr %v0, align 8
  %t16 = getelementptr %struct.String, ptr %t14, i32 0, i32 0
  %t15 = load ptr, ptr %t16
  %t17 = load i64, ptr %v2, align 8
  %t19 = getelementptr i8, ptr %t15, i64 %t17
  %t20 = load i8, ptr %t19
  %t18 = bitcast i8 %t20 to i8
  %t21 = getelementptr i8, ptr %t12, i64 %t13
  store i8 %t18, ptr %t21
  %t22 = load i64, ptr %v2, align 8
  %t23 = add i64 %t22, 1
  store i64 %t23, ptr %v2, align 8
  br label %while.cond0
  while.end2:
  %t24 = load ptr, ptr %v1, align 8
  %t25 = load ptr, ptr %v0, align 8
  %t27 = getelementptr %struct.String, ptr %t25, i32 0, i32 1
  %t26 = load i64, ptr %t27
  %t29 = alloca %struct.String, align 8
  %t30 = getelementptr %struct.String, ptr %t29, i32 0, i32 0
  store ptr %t24, ptr %t30
  %t31 = getelementptr %struct.String, ptr %t29, i32 0, i32 1
  store i64 %t26, ptr %t31
  %t28 = load %struct.String, ptr %t29
  %t33 = alloca %struct.String, align 8
  store %struct.String %t28, ptr %t33
  %l34 = call i8* @malloc(i64 16)
  call void @llvm.memcpy.p0.p0.i64(i8* %l34, ptr %t33, i64 16, i1 false)
  %t32 = bitcast i8* %l34 to ptr
  ret ptr %t32
}

