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
	subq	$104, %rsp
	.cfi_offset %rbx, -56
	.cfi_offset %r12, -48
	.cfi_offset %r13, -40
	.cfi_offset %r14, -32
	.cfi_offset %r15, -24
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -96(%rbp)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-96(%rbp), %rdi
	movq	%rdi, -136(%rbp)
	callq	inc_ref@PLT
	movl	$2, %edi
	callq	alloc_int@PLT
	movq	%rax, -104(%rbp)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-104(%rbp), %rdi
	movq	%rdi, -48(%rbp)
	callq	inc_ref@PLT
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, -56(%rbp)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-56(%rbp), %rdi
	movq	%rdi, -144(%rbp)
	callq	inc_ref@PLT
	movl	$5, %edi
	callq	alloc_int@PLT
	movq	%rax, -80(%rbp)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -88(%rbp)
	.p2align	4, 0x90
.LBB1_1:
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	-56(%rbp), %rdi
	movq	-80(%rbp), %rsi
	callq	less@PLT
	movq	%rax, -16(%r14)
	movq	%rax, %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB1_3
	movq	%rsp, %r15
	leaq	-16(%r15), %rax
	movq	%rax, -72(%rbp)
	movq	%rax, %rsp
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
	movq	-88(%rbp), %rsi
	callq	add@PLT
	movq	%rax, -56(%rbp)
	movq	-104(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -104(%rbp)
	movq	-16(%r14), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r14)
	jmp	.LBB1_1
.LBB1_3:
	movq	%rsp, %rax
	movq	%rax, -64(%rbp)
	leaq	-16(%rax), %r14
	movq	%r14, %rsp
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	%rsp, %r15
	leaq	-16(%r15), %rsp
	movq	%rsp, %r13
	leaq	-16(%r13), %rsp
	movq	.str.0@GOTPCREL(%rip), %rdi
	movl	$4, %esi
	callq	alloc_string@PLT
	movq	%rax, -16(%r13)
	movq	-48(%rbp), %rsi
	movq	%rsi, -16(%r15)
	movq	-16(%r13), %rdi
	callq	add@PLT
	movq	%rax, -16(%r12)
	movq	.str.1@GOTPCREL(%rip), %rdi
	movl	$1, %esi
	callq	alloc_string@PLT
	movq	%rax, -16(%rbx)
	movq	-16(%r12), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	-64(%rbp), %r15
	movq	%rax, -16(%r15)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-16(%r15), %rdi
	callq	write@PLT
	movq	-88(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -88(%rbp)
	movq	-80(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -80(%rbp)
	movq	-56(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -56(%rbp)
	movq	-16(%r13), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r13)
	movq	-16(%r12), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r12)
	movq	-16(%rbx), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%rbx)
	.p2align	4, 0x90
.LBB1_4:
	movq	%rsp, %r12
	leaq	-16(%r12), %r15
	movq	%r15, %rsp
	movq	%rsp, %r13
	leaq	-16(%r13), %rsp
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	-48(%rbp), %rax
	movq	%rax, -16(%rbx)
	movl	$10, %edi
	callq	alloc_int@PLT
	movq	%rax, -16(%r13)
	movq	-16(%rbx), %rdi
	movq	%rax, %rsi
	callq	less@PLT
	movq	%rax, -16(%r12)
	movq	(%r14), %rdi
	callq	del_obj@PLT
	movq	$0, (%r14)
	movq	-16(%r13), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r13)
	movq	-16(%r12), %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB1_6
	movq	%rsp, %r13
	leaq	-16(%r13), %rsp
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	-48(%rbp), %rax
	movq	%rax, -16(%rbx)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -16(%r12)
	movq	-16(%rbx), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	%rax, -16(%r13)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-16(%r13), %rax
	movq	%rax, -48(%rbp)
	movq	-16(%r13), %rdi
	callq	inc_ref@PLT
	movq	-72(%rbp), %rax
	movq	(%rax), %rdi
	callq	dec_ref@PLT
	movq	(%r15), %rdi
	callq	del_obj@PLT
	movq	$0, (%r15)
	movq	-16(%r12), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r12)
	jmp	.LBB1_4
.LBB1_6:
	movq	%rsp, %rax
	movq	%rax, -72(%rbp)
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -64(%rbp)
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -128(%rbp)
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	leaq	-16(%rax), %rsp
	movq	%rax, %r15
	movq	%rsp, %rax
	leaq	-16(%rax), %rsp
	movq	%rax, %r13
	movq	%rax, -120(%rbp)
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	.str.2@GOTPCREL(%rip), %rdi
	movl	$8, %esi
	callq	alloc_string@PLT
	movq	%rax, -16(%rbx)
	movq	-136(%rbp), %rdi
	movq	%rdi, -16(%r12)
	callq	inc_ref@PLT
	movq	-48(%rbp), %rdi
	movq	%rdi, -16(%r14)
	callq	inc_ref@PLT
	movq	-16(%r12), %rdi
	movq	-16(%r14), %rsi
	callq	myadd@PLT
	movq	%rax, -16(%r13)
	movq	-16(%rbx), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	%r15, -112(%rbp)
	movq	%rax, -16(%r15)
	movq	.str.3@GOTPCREL(%rip), %rdi
	movl	$1, %esi
	callq	alloc_string@PLT
	movq	-128(%rbp), %r13
	movq	%rax, -16(%r13)
	movq	-16(%r15), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	-64(%rbp), %r12
	movq	%rax, -16(%r12)
	movq	%rax, %rdi
	callq	write@PLT
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	-72(%rbp), %r14
	movq	%rax, -16(%r14)
	movq	-96(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -96(%rbp)
	movq	-120(%rbp), %r15
	movq	-16(%r15), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r15)
	movq	-16(%rbx), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%rbx)
	movq	-16(%r13), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r13)
	movq	-112(%rbp), %rbx
	movq	-16(%rbx), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%rbx)
	movq	-16(%r12), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r12)
	movq	-16(%r14), %rbx
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

	.type	.str.0,@object
	.section	.rodata,"a",@progbits
	.globl	.str.0
.str.0:
	.ascii	"b = "
	.size	.str.0, 4

	.type	.str.1,@object
	.globl	.str.1
.str.1:
	.byte	10
	.size	.str.1, 1

	.type	.str.2,@object
	.globl	.str.2
.str.2:
	.ascii	"a + b = "
	.size	.str.2, 8

	.type	.str.3,@object
	.globl	.str.3
.str.3:
	.byte	10
	.size	.str.3, 1

	.section	".note.GNU-stack","",@progbits
