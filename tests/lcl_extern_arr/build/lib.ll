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

define i64 @run_arr_char(ptr) {
  %v0 = alloca ptr, align 8
  %v1 = alloca i64, align 8
  store ptr %0, ptr %v0, align 8
  %t0 = load ptr, ptr %v0, align 8
  %t1 = call i64 @strlen(ptr %t0)
  store i64 %t1, ptr %v1, align 8
  %c1000000 = load ptr, ptr %v0, align 8
  call void @__ayanami_unique_free(i8* %c1000000)
  %t2 = load i64, ptr %v1, align 8
  ret i64 %t2
}

