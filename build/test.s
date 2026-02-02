	.text
	.file	"ayanami_modlue"
	.section	.rodata.cst8,"aM",@progbits,8
	.p2align	3, 0x0
.LCPI0_0:
	.quad	0x4000cccccccccccd
	.text
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
	movl	$2, %edi
	callq	alloc_int@PLT
	movq	%rax, -112(%rbp)
	movsd	.LCPI0_0(%rip), %xmm0
	callq	alloc_float@PLT
	movq	%rax, -120(%rbp)
	movl	$99, %edi
	callq	alloc_char@PLT
	movq	%rax, -128(%rbp)
	movq	.str.0@GOTPCREL(%rip), %rdi
	movl	$3, %esi
	callq	alloc_string@PLT
	movq	%rax, -136(%rbp)
	movl	$4, %edi
	callq	alloc_array@PLT
	movq	%rax, -80(%rbp)
	movl	$4, 8(%rax)
	movq	16(%rax), %rax
	movq	-112(%rbp), %rcx
	movq	%rcx, (%rax)
	movq	-120(%rbp), %rcx
	movq	%rcx, 8(%rax)
	movq	-128(%rbp), %rcx
	movq	%rcx, 16(%rax)
	movq	-136(%rbp), %rcx
	movq	%rcx, 24(%rax)
	movq	-80(%rbp), %rdi
	callq	inc_ref@PLT
	movq	-80(%rbp), %rax
	movq	%rax, -144(%rbp)
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, -48(%rbp)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-48(%rbp), %rax
	movq	%rax, -72(%rbp)
	movl	$4, %edi
	callq	alloc_int@PLT
	movq	%rax, -56(%rbp)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -64(%rbp)
	.p2align	4, 0x90
.LBB0_1:
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	-48(%rbp), %rdi
	movq	-56(%rbp), %rsi
	callq	less@PLT
	movq	%rax, -16(%rbx)
	movq	%rax, %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB0_3
	movq	%rsp, %rax
	movq	%rax, -104(%rbp)
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -96(%rbp)
	leaq	-16(%rax), %rsp
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %rax
	movq	%rax, -88(%rbp)
	leaq	-16(%rax), %rsp
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	%rsp, %r13
	leaq	-16(%r13), %rsp
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %r15
	leaq	-16(%r15), %rsp
	movq	-72(%rbp), %rax
	movq	%rax, -16(%r15)
	movq	.str.1@GOTPCREL(%rip), %rdi
	movl	$2, %esi
	callq	alloc_string@PLT
	movq	%rax, -16(%rbx)
	movq	-16(%r15), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	%rax, -16(%r13)
	movq	%rax, %rdi
	callq	write@PLT
	movq	-72(%rbp), %rax
	movq	%rax, -16(%r12)
	movq	-144(%rbp), %rax
	movq	-88(%rbp), %rcx
	movq	%rax, -16(%rcx)
	movq	16(%rax), %r15
	movq	-16(%r12), %rdi
	callq	get_int_value@PLT
	cltq
	movq	(%r15,%rax,8), %rax
	movq	%rax, -16(%r14)
	movq	.str.2@GOTPCREL(%rip), %rdi
	movl	$1, %esi
	callq	alloc_string@PLT
	movq	-96(%rbp), %r15
	movq	%rax, -16(%r15)
	movq	-16(%r14), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	-104(%rbp), %r14
	movq	%rax, -16(%r14)
	movq	%rax, %rdi
	callq	write@PLT
	movq	-48(%rbp), %rdi
	movq	-64(%rbp), %rsi
	callq	add@PLT
	movq	%rax, -48(%rbp)
	movq	%rax, -72(%rbp)
	movq	-16(%rbx), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%rbx)
	movq	-16(%r13), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r13)
	movq	-16(%r15), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r15)
	movq	-16(%r14), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r14)
	jmp	.LBB0_1
.LBB0_3:
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, -16(%rbx)
	movq	-48(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -48(%rbp)
	movq	-64(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -64(%rbp)
	movq	-56(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -56(%rbp)
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
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc

	.type	.str.0,@object
	.section	.rodata,"a",@progbits
	.globl	.str.0
.str.0:
	.ascii	"str"
	.size	.str.0, 3

	.type	.str.1,@object
	.globl	.str.1
.str.1:
	.ascii	": "
	.size	.str.1, 2

	.type	.str.2,@object
	.globl	.str.2
.str.2:
	.byte	32
	.size	.str.2, 1

	.section	".note.GNU-stack","",@progbits
