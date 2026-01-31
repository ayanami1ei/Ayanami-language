	.text
	.file	"ayanami_modlue"
	.globl	myadd                           # -- Begin function myadd
	.p2align	4, 0x90
	.type	myadd,@function
myadd:                                  # @myadd
	.cfi_startproc
# %bb.0:                                # %entry
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
	subq	$24, %rsp
	.cfi_offset %rbx, -56
	.cfi_offset %r12, -48
	.cfi_offset %r13, -40
	.cfi_offset %r14, -32
	.cfi_offset %r15, -24
	movq	%rdi, -56(%rbp)
	movq	%rsi, -48(%rbp)
	movq	%rsp, %rax
	leaq	-16(%rax), %r14
	movq	%r14, %rsp
	movq	%rsp, %rcx
	leaq	-16(%rcx), %r12
	movq	%r12, %rsp
	movq	-56(%rbp), %rdx
	movq	%rdx, -16(%rcx)
	movq	-48(%rbp), %rcx
	movq	%rcx, -16(%rax)
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %r15
	leaq	-16(%r15), %rsp
	movq	%rsp, %r13
	leaq	-16(%r13), %rsp
	movq	-56(%rbp), %rax
	movq	%rax, -16(%r13)
	movq	-48(%rbp), %rsi
	movq	%rsi, -16(%r15)
	movq	-16(%r13), %rdi
	callq	add@PLT
	movq	%rax, -16(%rbx)
	movq	(%r12), %rdi
	callq	dec_ref@PLT
	movq	(%r14), %rdi
	callq	dec_ref@PLT
	movq	-16(%r13), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r13)
	movq	-16(%r15), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r15)
	movq	-16(%rbx), %rbx
	movq	%rbx, %rdi
	callq	inc_ref@PLT
	movq	%rbx, %rax
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
	.size	myadd, .Lfunc_end0-myadd
	.cfi_endproc
                                        # -- End function
	.globl	main                            # -- Begin function main
	.p2align	4, 0x90
	.type	main,@function
main:                                   # @main
	.cfi_startproc
# %bb.0:                                # %entry
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
	movq	%rax, -72(%rbp)
	movq	%rax, -80(%rbp)
	movl	$2, %edi
	callq	alloc_int@PLT
	movq	%rax, -96(%rbp)
	movq	%rax, -88(%rbp)
	movq	%rax, %rdi
	callq	del_obj@PLT
	movq	$0, -96(%rbp)
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, -48(%rbp)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-48(%rbp), %rax
	movq	%rax, -104(%rbp)
	movl	$5, %edi
	callq	alloc_int@PLT
	movq	%rax, -56(%rbp)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -64(%rbp)
	.p2align	4, 0x90
.LBB1_1:                                # %block_5
                                        # =>This Inner Loop Header: Depth=1
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	-48(%rbp), %rdi
	movq	-56(%rbp), %rsi
	callq	less@PLT
	movq	%rax, -16(%rbx)
	movq	%rax, %rdi
	callq	is_truth@PLT
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %r15
	leaq	-16(%r15), %rsp
	testb	$1, %al
	je	.LBB1_3
# %bb.2:                                # %block_6
                                        #   in Loop: Header=BB1_1 Depth=1
	movq	-88(%rbp), %rax
	movq	%rax, -16(%r15)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -16(%r14)
	movq	-16(%r15), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	%rax, -16(%rbx)
	movq	%rax, -88(%rbp)
	movq	-96(%rbp), %rdi
	callq	dec_ref@PLT
	movq	-48(%rbp), %rdi
	movq	-64(%rbp), %rsi
	callq	add@PLT
	movq	%rax, -48(%rbp)
	movq	-16(%r15), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r15)
	movq	-16(%r14), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r14)
	movq	-16(%rbx), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%rbx)
	jmp	.LBB1_1
.LBB1_3:                                # %block_merge_7
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	%rsp, %r13
	leaq	-16(%r13), %rsp
	movq	-80(%rbp), %rax
	movq	%rax, -16(%r13)
	movq	-88(%rbp), %rsi
	movq	%rsi, -16(%r12)
	movq	-16(%r13), %rdi
	callq	myadd@PLT
	movq	%rax, -16(%r15)
	movq	%rax, -80(%rbp)
	movq	-72(%rbp), %rdi
	callq	dec_ref@PLT
	movq	-80(%rbp), %rdi
	movq	%rdi, -16(%r14)
	callq	write@PLT
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, -16(%rbx)
	movq	-16(%r13), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r13)
	movq	-56(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -56(%rbp)
	movq	-64(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -64(%rbp)
	movq	-48(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -48(%rbp)
	movq	-16(%r12), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r12)
	movq	-16(%r15), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r15)
	movq	-16(%r14), %rdi
	callq	del_obj@PLT
	movq	$0, -16(%r14)
	movq	-72(%rbp), %rdi
	callq	del_obj@PLT
	movq	$0, -72(%rbp)
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
                                        # -- End function
	.section	".note.GNU-stack","",@progbits
