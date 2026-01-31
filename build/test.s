	.text
	.file	"ayanami_modlue"
	.globl	myadd
	.p2align	4, 0x90
	.type	myadd,@function
myadd:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	pushq	%r14
	pushq	%rbx
	subq	$48, %rsp
	.cfi_offset %rbx, -32
	.cfi_offset %r14, -24
	movq	%rdi, -32(%rbp)
	movq	%rsi, -40(%rbp)
	movq	%rdi, -48(%rbp)
	movq	%rsi, -56(%rbp)
	callq	greater@PLT
	movq	%rax, -24(%rbp)
	movq	%rax, %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB0_3
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	-32(%rbp), %rax
	movq	%rax, -16(%rbx)
	movq	-24(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -24(%rbp)
	movq	-16(%rbx), %rbx
	movq	%rbx, %rdi
	jmp	.LBB0_2
.LBB0_3:
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %rax
	leaq	-16(%rax), %rsp
	movq	%rsp, %rcx
	leaq	-16(%rcx), %rsp
	movq	-32(%rbp), %rdx
	movq	%rdx, -16(%rcx)
	movq	-40(%rbp), %rsi
	movq	%rsi, -16(%rax)
	movq	-16(%rcx), %rdi
	callq	add@PLT
	movq	%rax, %rbx
	movq	%rax, -16(%r14)
	movq	%rax, %rdi
.LBB0_2:
	callq	inc_ref@PLT
	movq	%rbx, %rax
	leaq	-16(%rbp), %rsp
	popq	%rbx
	popq	%r14
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end0:
	.size	myadd, .Lfunc_end0-myadd
	.cfi_endproc

	.globl	main
	.p2align	4, 0x90
	.type	main,@function
main:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	pushq	%r15
	pushq	%r14
	pushq	%r13
	pushq	%r12
	pushq	%rbx
	subq	$72, %rsp
	.cfi_offset %rbx, -56
	.cfi_offset %r12, -48
	.cfi_offset %r13, -40
	.cfi_offset %r14, -32
	.cfi_offset %r15, -24
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -80(%rbp)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-80(%rbp), %rdi
	movq	%rdi, -96(%rbp)
	callq	inc_ref@PLT
	movl	$2, %edi
	callq	alloc_int@PLT
	movq	%rax, -88(%rbp)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-88(%rbp), %rdi
	movq	%rdi, -48(%rbp)
	callq	inc_ref@PLT
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, -56(%rbp)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-56(%rbp), %rdi
	movq	%rdi, -104(%rbp)
	callq	inc_ref@PLT
	movl	$5, %edi
	callq	alloc_int@PLT
	movq	%rax, -64(%rbp)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -72(%rbp)
	.p2align	4, 0x90
.LBB1_1:
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	-56(%rbp), %rdi
	movq	-64(%rbp), %rsi
	callq	less@PLT
	movq	%rax, -16(%r14)
	movq	%rax, %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB1_3
	movq	%rsp, %r15
	leaq	-16(%r15), %rbx
	movq	%rbx, %rsp
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	-48(%rbp), %rax
	movq	%rax, -16(%r12)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -16(%r14)
	movq	-16(%r12), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	%rax, -16(%r15)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-16(%r15), %rax
	movq	%rax, -48(%rbp)
	movq	-16(%r15), %rdi
	callq	inc_ref@PLT
	movq	-56(%rbp), %rdi
	movq	-72(%rbp), %rsi
	callq	add@PLT
	movq	%rax, -56(%rbp)
	movq	-88(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -88(%rbp)
	movq	-16(%r14), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r14)
	jmp	.LBB1_1
.LBB1_3:
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	-48(%rbp), %rdi
	movq	%rdi, -16(%r14)
	callq	inc_ref@PLT
	movq	-16(%r14), %rdi
	callq	write@PLT
	movq	-56(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -56(%rbp)
	movq	-64(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -64(%rbp)
	movq	-72(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -72(%rbp)
	.p2align	4, 0x90
.LBB1_4:
	movq	%rsp, %r15
	leaq	-16(%r15), %r14
	movq	%r14, %rsp
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	%rsp, %r13
	leaq	-16(%r13), %rsp
	movq	-48(%rbp), %rax
	movq	%rax, -16(%r13)
	movl	$10, %edi
	callq	alloc_int@PLT
	movq	%rax, -16(%r12)
	movq	-16(%r13), %rdi
	movq	%rax, %rsi
	callq	less@PLT
	movq	%rax, -16(%r15)
	movq	-16(%r12), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r12)
	movq	-16(%r15), %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB1_6
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	%rsp, %r15
	leaq	-16(%r15), %rsp
	movq	%rsp, %r13
	leaq	-16(%r13), %rsp
	movq	-48(%rbp), %rax
	movq	%rax, -16(%r13)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -16(%r15)
	movq	-16(%r13), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	%rax, -16(%r12)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-16(%r12), %rax
	movq	%rax, -48(%rbp)
	movq	-16(%r12), %rdi
	callq	inc_ref@PLT
	movq	(%rbx), %rdi
	callq	dec_ref@PLT
	movq	(%r14), %rdi
	callq	del_obj@PLT
	movq	$0, (%r14)
	movq	-16(%r15), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r15)
	jmp	.LBB1_4
.LBB1_6:
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %r15
	leaq	-16(%r15), %rsp
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	-96(%rbp), %rdi
	movq	%rdi, -16(%r12)
	callq	inc_ref@PLT
	movq	-48(%rbp), %rdi
	movq	%rdi, -16(%r15)
	callq	inc_ref@PLT
	movq	-16(%r12), %rdi
	movq	-16(%r15), %rsi
	callq	myadd@PLT
	movq	%rax, -16(%r14)
	movq	%rax, %rdi
	callq	write@PLT
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, -16(%rbx)
	movq	-80(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -80(%rbp)
	movq	-16(%r14), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r14)
	movq	-16(%rbx), %rbx
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	%rbx, %rdi
	callq	get_int_value@PLT
	movl	%eax, %r14d
	movq	%rbx, %rdi
	callq	del_obj@PLT
	movl	%r14d, %eax
	leaq	-40(%rbp), %rsp
	popq	%rbx
	popq	%r12
	popq	%r13
	popq	%r14
	popq	%r15
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end1:
	.size	main, .Lfunc_end1-main
	.cfi_endproc

	.section	".note.GNU-stack","",@progbits
