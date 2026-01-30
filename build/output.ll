; ModuleID = 'ayanami_modlue'
source_filename = "ayanami_modlue"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

declare void @err(ptr)

declare i32 @get_int_value(ptr)

declare i1 @is_truth(ptr)

declare void @write(ptr)

declare i1 @is_type(i64, ptr)

declare void @del_obj(ptr)

declare void @dec_ref(ptr)

declare void @inc_ref(ptr)

declare ptr @alloc_int(i64)

declare ptr @alloc_float(double)

declare ptr @alloc_bool(i1)

declare ptr @alloc_char(i32)

declare ptr @alloc_string(ptr, i32)

declare ptr @add(ptr, ptr)

declare ptr @sub(ptr, ptr)

declare ptr @mul(ptr, ptr)

declare ptr @div_op(ptr, ptr)

declare ptr @equal(ptr, ptr)

declare ptr @greater(ptr, ptr)

declare ptr @less(ptr, ptr)

declare ptr @greater_equal(ptr, ptr)

declare ptr @less_equal(ptr, ptr)

declare ptr @and_op(ptr, ptr)

declare ptr @or_op(ptr, ptr)

declare ptr @not(ptr)

define ptr @myadd(ptr %0, ptr %1) {
entry:
  %slotslot_2 = alloca ptr, align 8
  store ptr %0, ptr %slotslot_2, align 8
  %slotslot_3 = alloca ptr, align 8
  store ptr %1, ptr %slotslot_3, align 8
  %slotslot_4 = alloca ptr, align 8
  %left_obj = load ptr, ptr %slotslot_2, align 8
  %right_obj = load ptr, ptr %slotslot_3, align 8
  %add = call ptr @add(ptr %left_obj, ptr %right_obj)
  store ptr %add, ptr %slotslot_4, align 8
  %left_obj1 = load ptr, ptr %slotslot_2, align 8
  call void @dec_ref(ptr %left_obj1)
  %left_obj2 = load ptr, ptr %slotslot_3, align 8
  call void @dec_ref(ptr %left_obj2)
  %ret.load = load ptr, ptr %slotslot_4, align 8
  call void @inc_ref(ptr %ret.load)
  ret ptr %ret.load
}

define i32 @main() {
entry:
  %"new int" = call ptr @alloc_int(i64 1)
  %slotslot_6 = alloca ptr, align 8
  store ptr %"new int", ptr %slotslot_6, align 8
  %left_obj = load ptr, ptr %slotslot_6, align 8
  call void @inc_ref(ptr %left_obj)
  %slotslot_61 = alloca ptr, align 8
  %bind.load = load ptr, ptr %slotslot_6, align 8
  store ptr %bind.load, ptr %slotslot_61, align 8
  %"new int2" = call ptr @alloc_int(i64 2)
  %slotslot_7 = alloca ptr, align 8
  store ptr %"new int2", ptr %slotslot_7, align 8
  %left_obj3 = load ptr, ptr %slotslot_7, align 8
  call void @inc_ref(ptr %left_obj3)
  %slotslot_74 = alloca ptr, align 8
  %bind.load5 = load ptr, ptr %slotslot_7, align 8
  store ptr %bind.load5, ptr %slotslot_74, align 8
  %"new int6" = call ptr @alloc_int(i64 0)
  %slotslot_8 = alloca ptr, align 8
  store ptr %"new int6", ptr %slotslot_8, align 8
  %left_obj7 = load ptr, ptr %slotslot_8, align 8
  call void @inc_ref(ptr %left_obj7)
  %slotslot_88 = alloca ptr, align 8
  %bind.load9 = load ptr, ptr %slotslot_8, align 8
  store ptr %bind.load9, ptr %slotslot_88, align 8
  br label %block_5

block_5:                                          ; preds = %block_6, %entry
  %"new int10" = call ptr @alloc_int(i64 5)
  %slotslot_9 = alloca ptr, align 8
  store ptr %"new int10", ptr %slotslot_9, align 8
  %"new int11" = call ptr @alloc_int(i64 1)
  %slotslot_10 = alloca ptr, align 8
  store ptr %"new int11", ptr %slotslot_10, align 8
  %slotslot_11 = alloca ptr, align 8
  %left_obj12 = load ptr, ptr %slotslot_8, align 8
  %right_obj = load ptr, ptr %slotslot_9, align 8
  %less = call ptr @less(ptr %left_obj12, ptr %right_obj)
  store ptr %less, ptr %slotslot_11, align 8
  %cond_load = load ptr, ptr %slotslot_11, align 8
  %"call is_truth" = call i1 @is_truth(ptr %cond_load)
  br i1 %"call is_truth", label %block_6, label %block_merge_7

block_6:                                          ; preds = %block_5
  %"new int13" = call ptr @alloc_int(i64 1)
  %slotslot_12 = alloca ptr, align 8
  store ptr %"new int13", ptr %slotslot_12, align 8
  %slotslot_13 = alloca ptr, align 8
  %left_obj14 = load ptr, ptr %slotslot_74, align 8
  %right_obj15 = load ptr, ptr %slotslot_12, align 8
  %add = call ptr @add(ptr %left_obj14, ptr %right_obj15)
  store ptr %add, ptr %slotslot_13, align 8
  %left_obj16 = load ptr, ptr %slotslot_13, align 8
  call void @inc_ref(ptr %left_obj16)
  %slotslot_1317 = alloca ptr, align 8
  %bind.load18 = load ptr, ptr %slotslot_13, align 8
  store ptr %bind.load18, ptr %slotslot_1317, align 8
  %left_obj19 = load ptr, ptr %slotslot_8, align 8
  %right_obj20 = load ptr, ptr %slotslot_10, align 8
  %add21 = call ptr @add(ptr %left_obj19, ptr %right_obj20)
  store ptr %add21, ptr %slotslot_8, align 8
  %load = load ptr, ptr %slotslot_12, align 8
  call void @del_obj(ptr %load)
  store ptr null, ptr %slotslot_12, align 8
  %load22 = load ptr, ptr %slotslot_74, align 8
  call void @del_obj(ptr %load22)
  store ptr null, ptr %slotslot_74, align 8
  br label %block_5

block_merge_7:                                    ; preds = %block_5
  %slotslot_623 = alloca ptr, align 8
  %bind.load24 = load ptr, ptr %slotslot_61, align 8
  store ptr %bind.load24, ptr %slotslot_623, align 8
  %slotslot_1325 = alloca ptr, align 8
  %bind.load26 = load ptr, ptr %slotslot_1317, align 8
  store ptr %bind.load26, ptr %slotslot_1325, align 8
  %param = load ptr, ptr %slotslot_623, align 8
  %param27 = load ptr, ptr %slotslot_1325, align 8
  %call = call ptr @myadd(ptr %param, ptr %param27)
  %slotslot_1 = alloca ptr, align 8
  store ptr %call, ptr %slotslot_1, align 8
  %slotslot_128 = alloca ptr, align 8
  %bind.load29 = load ptr, ptr %slotslot_1, align 8
  store ptr %bind.load29, ptr %slotslot_128, align 8
  %slotslot_130 = alloca ptr, align 8
  %bind.load31 = load ptr, ptr %slotslot_128, align 8
  store ptr %bind.load31, ptr %slotslot_130, align 8
  %param32 = load ptr, ptr %slotslot_130, align 8
  call void @write(ptr %param32)
  %"new int33" = call ptr @alloc_int(i64 0)
  %slotslot_14 = alloca ptr, align 8
  store ptr %"new int33", ptr %slotslot_14, align 8
  %load34 = load ptr, ptr %slotslot_9, align 8
  call void @del_obj(ptr %load34)
  store ptr null, ptr %slotslot_9, align 8
  %load35 = load ptr, ptr %slotslot_10, align 8
  call void @del_obj(ptr %load35)
  store ptr null, ptr %slotslot_10, align 8
  %load36 = load ptr, ptr %slotslot_8, align 8
  call void @del_obj(ptr %load36)
  store ptr null, ptr %slotslot_8, align 8
  %load37 = load ptr, ptr %slotslot_1317, align 8
  call void @del_obj(ptr %load37)
  store ptr null, ptr %slotslot_1317, align 8
  %load38 = load ptr, ptr %slotslot_128, align 8
  call void @del_obj(ptr %load38)
  store ptr null, ptr %slotslot_128, align 8
  %load39 = load ptr, ptr %slotslot_61, align 8
  call void @del_obj(ptr %load39)
  store ptr null, ptr %slotslot_61, align 8
  %ret.load = load ptr, ptr %slotslot_14, align 8
  call void @inc_ref(ptr %ret.load)
  %"get main ret" = call i32 @get_int_value(ptr %ret.load)
  call void @del_obj(ptr %ret.load)
  ret i32 %"get main ret"
}
