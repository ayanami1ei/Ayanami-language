; ModuleID = 'ayanami'
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

%struct.String = type { ptr, i64 }

declare noalias i8* @__ayanami_unique_alloc(i64) allocsize(0) nounwind
declare void @__ayanami_unique_free(i8*) nounwind
declare void @llvm.memcpy.p0.p0.i64(i8*, i8*, i64, i1)
declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)
declare void @llvm.assume(i1)
declare void @__ayanami_require_fail(i64, i64) noreturn cold nounwind
declare void @__ayanami_ensure_fail(i64, i64) noreturn cold nounwind
declare void @__ayanami_invariant_fail(i64, i64) noreturn cold nounwind

declare i64 @strlen(ptr)
declare i64 @run_arr_char(ptr)

define i64 @main() {
  %v0 = alloca ptr, align 8
  %v1 = alloca ptr, align 8
  %v2 = alloca i64, align 8
  %v3 = alloca i64, align 8
  %v4 = alloca i64, align 8
  %v5 = alloca i64, align 8
  %t1 = call i8* @__ayanami_unique_alloc(i64 3)
  %t0 = bitcast i8* %t1 to ptr
  %t2 = getelementptr i8, ptr %t0, i64 0
  store i8 104, ptr %t2
  %t3 = getelementptr i8, ptr %t0, i64 1
  store i8 105, ptr %t3
  %t4 = getelementptr i8, ptr %t0, i64 2
  store i8 0, ptr %t4
  store ptr %t0, ptr %v0, align 8
  %t5 = load ptr, ptr %v0, align 8
  %t6 = call i64 @strlen(ptr %t5)
  %t7 = icmp ne i64 %t6, 2
  br i1 %t7, label %then0, label %else1
  then0:
  store i64 1, ptr %v2, align 8
  %c1000000 = load ptr, ptr %v0, align 8
  call void @__ayanami_unique_free(i8* %c1000000)
  %t8 = load i64, ptr %v2, align 8
  ret i64 %t8
  br label %ifcont2
  else1:
  br label %ifcont2
  ifcont2:
  %t10 = call i8* @__ayanami_unique_alloc(i64 3)
  %t9 = bitcast i8* %t10 to ptr
  %t11 = getelementptr i8, ptr %t9, i64 0
  store i8 111, ptr %t11
  %t12 = getelementptr i8, ptr %t9, i64 1
  store i8 107, ptr %t12
  %t13 = getelementptr i8, ptr %t9, i64 2
  store i8 0, ptr %t13
  store ptr %t9, ptr %v1, align 8
  %t14 = load ptr, ptr %v1
  store ptr zeroinitializer, ptr %v1
  %t15 = call i64 @run_arr_char(ptr %t14)
  %t16 = icmp ne i64 %t15, 2
  br i1 %t16, label %then3, label %else4
  then3:
  store i64 2, ptr %v3, align 8
  %c1000001 = load ptr, ptr %v0, align 8
  call void @__ayanami_unique_free(i8* %c1000001)
  %t17 = load i64, ptr %v3, align 8
  ret i64 %t17
  br label %ifcont5
  else4:
  br label %ifcont5
  ifcont5:
  store i64 0, ptr %v4, align 8
  %c1000002 = load ptr, ptr %v0, align 8
  call void @__ayanami_unique_free(i8* %c1000002)
  %t18 = load i64, ptr %v4, align 8
  ret i64 %t18
  store i64 0, ptr %v5, align 8
  %t19 = load i64, ptr %v5, align 8
  ret i64 %t19
}

