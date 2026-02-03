	.text
	.file	"ayanami_modlue"
	.globl	user_partition_5
	.p2align	4, 0x90
	.type	user_partition_5,@function
user_partition_5:
	.cfi_startproc
	pushq	%rbx
	.cfi_def_cfa_offset 16
	subq	$400, %rsp
	.cfi_def_cfa_offset 416
	.cfi_offset %rbx, -16
	movq	%rsi, %rbx
	movq	$0, 392(%rsp)
	movq	$0, 384(%rsp)
	movq	$0, 192(%rsp)
	movq	$0, 184(%rsp)
	movq	$0, 376(%rsp)
	movq	$0, 176(%rsp)
	movq	$0, 168(%rsp)
	movq	$0, 368(%rsp)
	movq	$0, 360(%rsp)
	movq	$0, 160(%rsp)
	movq	$0, 344(%rsp)
	movq	$0, 336(%rsp)
	movq	$0, 152(%rsp)
	movq	$0, 56(%rsp)
	movq	$0, 144(%rsp)
	movq	$0, 136(%rsp)
	movq	$0, 48(%rsp)
	movq	$0, 128(%rsp)
	movq	$0, 328(%rsp)
	movq	$0, 120(%rsp)
	movq	$0, 112(%rsp)
	movq	$0, 320(%rsp)
	movq	$0, 104(%rsp)
	movq	$0, 96(%rsp)
	movq	$0, 312(%rsp)
	movq	$0, 304(%rsp)
	movq	$0, 88(%rsp)
	movq	$0, 288(%rsp)
	movq	$0, 280(%rsp)
	movq	$0, 40(%rsp)
	movq	$0, 272(%rsp)
	movq	$0, 264(%rsp)
	movq	$0, 256(%rsp)
	movq	$0, 248(%rsp)
	movq	$0, 32(%rsp)
	movq	$0, 240(%rsp)
	movq	$0, 232(%rsp)
	movq	$0, 80(%rsp)
	movq	$0, 72(%rsp)
	movq	$0, 216(%rsp)
	movq	$0, 208(%rsp)
	movq	$0, 64(%rsp)
	movq	%rdx, 24(%rsp)
	movq	%rsi, 200(%rsp)
	movq	%rdi, (%rsp)
	movq	%rsi, 64(%rsp)
	movl	$1, %edi
	movl	$3, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	64(%rsp), %rax
	movq	%rax, 8(%rsp)
	movq	24(%rsp), %rdi
	movq	%rdi, 208(%rsp)
	movq	(%rsp), %rax
	movq	%rax, 216(%rsp)
	movq	16(%rax), %rbx
	callq	get_int_value@PLT
	cltq
	movq	(%rbx,%rax,8), %rbx
	movq	%rbx, 72(%rsp)
	movl	$1, %edi
	movl	$6, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	72(%rsp), %rax
	movq	%rax, 224(%rsp)
	movq	200(%rsp), %rbx
	movq	%rbx, 80(%rsp)
	movl	$1, %edi
	movl	$7, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	80(%rsp), %rax
	movq	%rax, 16(%rsp)
	jmp	.LBB0_1
	.p2align	4, 0x90
.LBB0_4:
	movq	16(%rsp), %rax
	movq	%rax, 144(%rsp)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, 56(%rsp)
	movq	144(%rsp), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	%rax, %rbx
	movq	%rax, 152(%rsp)
	movl	$1, %edi
	movl	$32, %esi
	movq	%rax, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	152(%rsp), %rax
	movq	%rax, 16(%rsp)
	movq	56(%rsp), %rbx
	xorl	%edi, %edi
	movl	$31, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 56(%rsp)
.LBB0_1:
	movq	16(%rsp), %rdi
	movq	%rdi, 232(%rsp)
	movq	24(%rsp), %rsi
	movq	%rsi, 240(%rsp)
	callq	less@PLT
	movq	%rax, 32(%rsp)
	movq	%rax, %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB0_5
	movq	32(%rsp), %rbx
	xorl	%edi, %edi
	movl	$10, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 32(%rsp)
	movq	16(%rsp), %rdi
	movq	%rdi, 248(%rsp)
	movq	(%rsp), %rax
	movq	%rax, 256(%rsp)
	movq	16(%rax), %rbx
	callq	get_int_value@PLT
	cltq
	movq	(%rbx,%rax,8), %rdi
	movq	%rdi, 264(%rsp)
	movq	224(%rsp), %rsi
	movq	%rsi, 272(%rsp)
	callq	less@PLT
	movq	%rax, 40(%rsp)
	movq	%rax, %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB0_4
	movq	8(%rsp), %rdi
	movq	%rdi, 280(%rsp)
	movq	(%rsp), %rax
	movq	%rax, 288(%rsp)
	movq	16(%rax), %rbx
	callq	get_int_value@PLT
	cltq
	movq	(%rbx,%rax,8), %rbx
	movq	%rbx, 88(%rsp)
	movl	$1, %edi
	movl	$18, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	88(%rsp), %rax
	movq	%rax, 296(%rsp)
	movq	16(%rsp), %rdi
	movq	%rdi, 304(%rsp)
	movq	(%rsp), %rax
	movq	%rax, 312(%rsp)
	movq	16(%rax), %rbx
	callq	get_int_value@PLT
	cltq
	movq	(%rbx,%rax,8), %rbx
	movq	%rbx, 96(%rsp)
	movq	8(%rsp), %rax
	movq	%rax, 104(%rsp)
	movl	$1, %edi
	movl	$21, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	(%rsp), %rax
	movq	%rax, 320(%rsp)
	movq	16(%rax), %rbx
	movq	104(%rsp), %rdi
	callq	get_int_value@PLT
	cltq
	movq	96(%rsp), %rcx
	movq	%rcx, (%rbx,%rax,8)
	movq	296(%rsp), %rbx
	movq	%rbx, 112(%rsp)
	movq	16(%rsp), %rax
	movq	%rax, 120(%rsp)
	movl	$1, %edi
	movl	$24, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	(%rsp), %rax
	movq	%rax, 328(%rsp)
	movq	16(%rax), %rbx
	movq	120(%rsp), %rdi
	callq	get_int_value@PLT
	cltq
	movq	112(%rsp), %rcx
	movq	%rcx, (%rbx,%rax,8)
	movq	8(%rsp), %rax
	movq	%rax, 128(%rsp)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, 48(%rsp)
	movq	128(%rsp), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	%rax, %rbx
	movq	%rax, 136(%rsp)
	movl	$1, %edi
	movl	$29, %esi
	movq	%rax, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	136(%rsp), %rax
	movq	%rax, 8(%rsp)
	movq	40(%rsp), %rbx
	xorl	%edi, %edi
	movl	$15, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 40(%rsp)
	movq	48(%rsp), %rbx
	xorl	%edi, %edi
	movl	$28, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 48(%rsp)
	jmp	.LBB0_4
.LBB0_5:
	movq	8(%rsp), %rdi
	movq	%rdi, 336(%rsp)
	movq	(%rsp), %rax
	movq	%rax, 344(%rsp)
	movq	16(%rax), %rbx
	callq	get_int_value@PLT
	cltq
	movq	(%rbx,%rax,8), %rbx
	movq	%rbx, 160(%rsp)
	movl	$1, %edi
	movl	$35, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	160(%rsp), %rax
	movq	%rax, 352(%rsp)
	movq	24(%rsp), %rdi
	movq	%rdi, 360(%rsp)
	movq	(%rsp), %rax
	movq	%rax, 368(%rsp)
	movq	16(%rax), %rbx
	callq	get_int_value@PLT
	cltq
	movq	(%rbx,%rax,8), %rbx
	movq	%rbx, 168(%rsp)
	movq	8(%rsp), %rax
	movq	%rax, 176(%rsp)
	movl	$1, %edi
	movl	$38, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	(%rsp), %rax
	movq	%rax, 376(%rsp)
	movq	16(%rax), %rbx
	movq	176(%rsp), %rdi
	callq	get_int_value@PLT
	cltq
	movq	168(%rsp), %rcx
	movq	%rcx, (%rbx,%rax,8)
	movq	352(%rsp), %rbx
	movq	%rbx, 184(%rsp)
	movq	24(%rsp), %rax
	movq	%rax, 192(%rsp)
	movl	$1, %edi
	movl	$41, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	(%rsp), %rax
	movq	%rax, 384(%rsp)
	movq	16(%rax), %rbx
	movq	192(%rsp), %rdi
	callq	get_int_value@PLT
	cltq
	movq	184(%rsp), %rcx
	movq	%rcx, (%rbx,%rax,8)
	movq	8(%rsp), %rbx
	movq	%rbx, 392(%rsp)
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	%rbx, %rax
	addq	$400, %rsp
	.cfi_def_cfa_offset 16
	popq	%rbx
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end0:
	.size	user_partition_5, .Lfunc_end0-user_partition_5
	.cfi_endproc

	.globl	user_qsort_17
	.p2align	4, 0x90
	.type	user_qsort_17,@function
user_qsort_17:
	.cfi_startproc
	pushq	%rbx
	.cfi_def_cfa_offset 16
	subq	$224, %rsp
	.cfi_def_cfa_offset 240
	.cfi_offset %rbx, -16
	movq	$0, 200(%rsp)
	movq	$0, 72(%rsp)
	movq	$0, 192(%rsp)
	movq	$0, 184(%rsp)
	movq	$0, 64(%rsp)
	movq	$0, 176(%rsp)
	movq	$0, 168(%rsp)
	movq	$0, 56(%rsp)
	movq	$0, 160(%rsp)
	movq	$0, 48(%rsp)
	movq	$0, 152(%rsp)
	movq	$0, 144(%rsp)
	movq	$0, 136(%rsp)
	movq	$0, 16(%rsp)
	movq	$0, 40(%rsp)
	movq	$0, 128(%rsp)
	movq	$0, 120(%rsp)
	movq	$0, 112(%rsp)
	movq	$0, 104(%rsp)
	movq	$0, 96(%rsp)
	movq	$0, 24(%rsp)
	movq	%rdx, 88(%rsp)
	movq	%rsi, 80(%rsp)
	movq	%rdi, 8(%rsp)
	movq	%rsi, 208(%rsp)
	movq	%rdx, 216(%rsp)
	movq	%rsi, %rdi
	movq	%rdx, %rsi
	callq	less@PLT
	movq	%rax, 24(%rsp)
	movq	%rax, %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB1_4
	movq	8(%rsp), %rbx
	movq	%rbx, 96(%rsp)
	movl	$1, %edi
	movl	$51, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	80(%rsp), %rbx
	movq	%rbx, 104(%rsp)
	movl	$1, %edi
	movl	$52, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	88(%rsp), %rbx
	movq	%rbx, 112(%rsp)
	movl	$1, %edi
	movl	$53, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	96(%rsp), %rdi
	movq	104(%rsp), %rsi
	movq	112(%rsp), %rdx
	callq	user_partition_5@PLT
	movq	%rax, %rbx
	movq	%rax, 120(%rsp)
	movl	$1, %edi
	movl	$54, %esi
	movq	%rax, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	120(%rsp), %rax
	movq	%rax, 32(%rsp)
	movq	24(%rsp), %rbx
	xorl	%edi, %edi
	movl	$50, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 24(%rsp)
	movq	32(%rsp), %rax
	movq	%rax, 128(%rsp)
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, 40(%rsp)
	movq	128(%rsp), %rdi
	movq	%rax, %rsi
	callq	greater@PLT
	movq	%rax, 16(%rsp)
	movq	40(%rsp), %rbx
	xorl	%edi, %edi
	movl	$56, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 40(%rsp)
	movq	16(%rsp), %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB1_3
	movq	8(%rsp), %rbx
	movq	%rbx, 136(%rsp)
	movl	$1, %edi
	movl	$59, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	80(%rsp), %rbx
	movq	%rbx, 144(%rsp)
	movl	$1, %edi
	movl	$60, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	32(%rsp), %rax
	movq	%rax, 152(%rsp)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, 48(%rsp)
	movq	152(%rsp), %rdi
	movq	%rax, %rsi
	callq	sub@PLT
	movq	%rax, %rbx
	movq	%rax, 160(%rsp)
	movl	$1, %edi
	movl	$63, %esi
	movq	%rax, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	136(%rsp), %rdi
	movq	144(%rsp), %rsi
	movq	160(%rsp), %rdx
	callq	user_qsort_17@PLT
	movq	%rax, 56(%rsp)
	movq	16(%rsp), %rbx
	xorl	%edi, %edi
	movl	$57, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 16(%rsp)
	movq	48(%rsp), %rbx
	xorl	%edi, %edi
	movl	$62, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 48(%rsp)
.LBB1_3:
	movq	8(%rsp), %rbx
	movq	%rbx, 168(%rsp)
	movl	$1, %edi
	movl	$65, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	32(%rsp), %rax
	movq	%rax, 176(%rsp)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, 64(%rsp)
	movq	176(%rsp), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	%rax, %rbx
	movq	%rax, 184(%rsp)
	movl	$1, %edi
	movl	$68, %esi
	movq	%rax, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	88(%rsp), %rbx
	movq	%rbx, 192(%rsp)
	movl	$1, %edi
	movl	$69, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	168(%rsp), %rdi
	movq	184(%rsp), %rsi
	movq	192(%rsp), %rdx
	callq	user_qsort_17@PLT
	movq	%rax, 72(%rsp)
	movq	56(%rsp), %rbx
	xorl	%edi, %edi
	movl	$58, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 56(%rsp)
	movq	64(%rsp), %rbx
	xorl	%edi, %edi
	movl	$67, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 64(%rsp)
.LBB1_4:
	movq	8(%rsp), %rax
	movq	%rax, 200(%rsp)
	movq	72(%rsp), %rbx
	xorl	%edi, %edi
	movl	$64, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 72(%rsp)
	movq	200(%rsp), %rbx
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	%rbx, %rax
	addq	$224, %rsp
	.cfi_def_cfa_offset 16
	popq	%rbx
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end1:
	.size	user_qsort_17, .Lfunc_end1-user_qsort_17
	.cfi_endproc

	.globl	main
	.p2align	4, 0x90
	.type	main,@function
main:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	pushq	%rbx
	.cfi_def_cfa_offset 24
	subq	$232, %rsp
	.cfi_def_cfa_offset 256
	.cfi_offset %rbx, -24
	.cfi_offset %rbp, -16
	movq	$0, 224(%rsp)
	movq	$0, 184(%rsp)
	movq	$0, 40(%rsp)
	movq	$0, 176(%rsp)
	movq	$0, 168(%rsp)
	movq	$0, 216(%rsp)
	movq	$0, 208(%rsp)
	movq	$0, 32(%rsp)
	movq	$0, 200(%rsp)
	movq	$0, 192(%rsp)
	movq	$0, 160(%rsp)
	movq	$0, 24(%rsp)
	movq	$0, 152(%rsp)
	movq	$0, 16(%rsp)
	movq	$0, 144(%rsp)
	movq	$0, 136(%rsp)
	movq	$0, 128(%rsp)
	movq	$0, 112(%rsp)
	movq	$0, 8(%rsp)
	movq	$0, 96(%rsp)
	movq	$0, 88(%rsp)
	movq	$0, 80(%rsp)
	movq	$0, 72(%rsp)
	movq	$0, 64(%rsp)
	movq	$0, 56(%rsp)
	movq	$0, 48(%rsp)
	movl	$5, %edi
	callq	alloc_int@PLT
	movq	%rax, 48(%rsp)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, 56(%rsp)
	movl	$4, %edi
	callq	alloc_int@PLT
	movq	%rax, 64(%rsp)
	movl	$2, %edi
	callq	alloc_int@PLT
	movq	%rax, 72(%rsp)
	movl	$8, %edi
	callq	alloc_int@PLT
	movq	%rax, 80(%rsp)
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, 88(%rsp)
	movl	$3, %edi
	callq	alloc_int@PLT
	movq	%rax, 96(%rsp)
	movl	$7, %edi
	callq	alloc_array@PLT
	movq	%rax, 8(%rsp)
	movl	$7, 8(%rax)
	movq	16(%rax), %rax
	movq	48(%rsp), %rcx
	movq	%rcx, (%rax)
	movq	56(%rsp), %rcx
	movq	%rcx, 8(%rax)
	movq	64(%rsp), %rcx
	movq	%rcx, 16(%rax)
	movq	72(%rsp), %rcx
	movq	%rcx, 24(%rax)
	movq	80(%rsp), %rcx
	movq	%rcx, 32(%rax)
	movq	88(%rsp), %rcx
	movq	%rcx, 40(%rax)
	movq	96(%rsp), %rcx
	movq	%rcx, 48(%rax)
	movq	8(%rsp), %rbx
	movl	$1, %edi
	movl	$78, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	8(%rsp), %rax
	movq	%rax, 104(%rsp)
	movl	$7, %edi
	callq	alloc_int@PLT
	movq	%rax, %rbx
	movq	%rax, 112(%rsp)
	movl	$1, %edi
	movl	$79, %esi
	movq	%rax, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	112(%rsp), %rax
	movq	%rax, 120(%rsp)
	movq	104(%rsp), %rbx
	movq	%rbx, 128(%rsp)
	movl	$1, %edi
	movl	$81, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, %rbx
	movq	%rax, 136(%rsp)
	movl	$1, %edi
	movl	$82, %esi
	movq	%rax, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	120(%rsp), %rax
	movq	%rax, 144(%rsp)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, 16(%rsp)
	movq	144(%rsp), %rdi
	movq	%rax, %rsi
	callq	sub@PLT
	movq	%rax, %rbx
	movq	%rax, 152(%rsp)
	movl	$1, %edi
	movl	$85, %esi
	movq	%rax, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	128(%rsp), %rdi
	movq	136(%rsp), %rsi
	movq	152(%rsp), %rdx
	callq	user_qsort_17@PLT
	movq	%rax, 24(%rsp)
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, %rbx
	movq	%rax, 160(%rsp)
	movl	$1, %edi
	movl	$86, %esi
	movq	%rax, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	160(%rsp), %rax
	movq	%rax, (%rsp)
	movq	16(%rsp), %rbx
	xorl	%edi, %edi
	movl	$84, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 16(%rsp)
	movq	24(%rsp), %rbx
	xorl	%edi, %edi
	movl	$80, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 24(%rsp)
	.p2align	4, 0x90
.LBB2_1:
	movq	(%rsp), %rdi
	movq	%rdi, 192(%rsp)
	movq	120(%rsp), %rsi
	movq	%rsi, 200(%rsp)
	callq	less@PLT
	movq	%rax, 32(%rsp)
	movq	%rax, %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB2_3
	movq	(%rsp), %rdi
	movq	%rdi, 208(%rsp)
	movq	104(%rsp), %rax
	movq	%rax, 216(%rsp)
	movq	16(%rax), %rbx
	callq	get_int_value@PLT
	cltq
	movq	(%rbx,%rax,8), %rbx
	movq	%rbx, 168(%rsp)
	movl	$1, %edi
	movl	$93, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	168(%rsp), %rdi
	callq	write@PLT
	movq	(%rsp), %rax
	movq	%rax, 176(%rsp)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, 40(%rsp)
	movq	176(%rsp), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	%rax, %rbx
	movq	%rax, 184(%rsp)
	movl	$1, %edi
	movl	$96, %esi
	movq	%rax, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	184(%rsp), %rax
	movq	%rax, (%rsp)
	movq	32(%rsp), %rbx
	xorl	%edi, %edi
	movl	$89, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 32(%rsp)
	movq	40(%rsp), %rbx
	xorl	%edi, %edi
	movl	$95, %esi
	movq	%rbx, %rdx
	callq	runtime_debug_ref@PLT
	movq	%rbx, %rdi
	callq	dec_ref@PLT
	movq	$0, 40(%rsp)
	jmp	.LBB2_1
.LBB2_3:
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, %rbx
	movq	%rax, 224(%rsp)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	%rbx, %rdi
	callq	get_int_value@PLT
	movl	%eax, %ebp
	movq	%rbx, %rdi
	callq	del_obj@PLT
	movl	%ebp, %eax
	addq	$232, %rsp
	.cfi_def_cfa_offset 24
	popq	%rbx
	.cfi_def_cfa_offset 16
	popq	%rbp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end2:
	.size	main, .Lfunc_end2-main
	.cfi_endproc

	.section	".note.GNU-stack","",@progbits
