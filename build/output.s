	.text
	.file	"ayanami_modlue"
	.globl	myadd                           # -- Begin function myadd
	.p2align	4, 0x90
	.type	myadd,@function
myadd:                                  # @myadd
	.cfi_startproc
# %bb.0:                                # %block_0
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	subq	$32, %rsp
# %bb.1:                                # %block_1
	movq	%rsp, %rax
	addq	$-16, %rax
	movq	%rax, -32(%rbp)                 # 8-byte Spill
	movq	%rax, %rsp
	movq	-8(%rbp), %rdi
	movq	-16(%rbp), %rsi
	callq	add@PLT
	movq	%rax, %rcx
	movq	-32(%rbp), %rax                 # 8-byte Reload
	movq	%rcx, (%rax)
	movq	-8(%rbp), %rdi
	callq	dec_ref@PLT
	movq	-16(%rbp), %rdi
	callq	dec_ref@PLT
	movq	-32(%rbp), %rax                 # 8-byte Reload
	movq	(%rax), %rdi
	movq	%rdi, -24(%rbp)                 # 8-byte Spill
	callq	inc_ref@PLT
	movq	-24(%rbp), %rax                 # 8-byte Reload
	movq	%rbp, %rsp
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
# %bb.0:                                # %block_2
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	subq	$64, %rsp
	jmp	.LBB1_1
.LBB1_1:                                # %block_3
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, %rcx
	movq	%rsp, %rax
	movq	%rax, -56(%rbp)                 # 8-byte Spill
	movq	%rax, %rdx
	addq	$-16, %rdx
	movq	%rdx, %rsp
	movq	%rcx, -16(%rax)
	movq	-16(%rax), %rdi
	callq	inc_ref@PLT
	movq	-56(%rbp), %rcx                 # 8-byte Reload
	movq	%rsp, %rax
	movq	%rax, -40(%rbp)                 # 8-byte Spill
	movq	%rax, %rdx
	addq	$-16, %rdx
	movq	%rdx, %rsp
	movq	-16(%rcx), %rcx
	movq	%rcx, -16(%rax)
	movl	$2, %edi
	callq	alloc_int@PLT
	movq	%rax, %rcx
	movq	%rsp, %rax
	movq	%rax, -48(%rbp)                 # 8-byte Spill
	movq	%rax, %rdx
	addq	$-16, %rdx
	movq	%rdx, %rsp
	movq	%rcx, -16(%rax)
	movq	-16(%rax), %rdi
	callq	inc_ref@PLT
	movq	-48(%rbp), %rdx                 # 8-byte Reload
	movq	-40(%rbp), %rcx                 # 8-byte Reload
	movq	%rsp, %rax
	movq	%rax, %rsi
	addq	$-16, %rsi
	movq	%rsi, %rsp
	movq	-16(%rdx), %rdx
	movq	%rdx, -16(%rax)
	movq	-16(%rcx), %rdi
	movq	-16(%rax), %rsi
	callq	myadd@PLT
	movq	%rsp, %rcx
	movq	%rcx, %rdx
	addq	$-16, %rdx
	movq	%rdx, -32(%rbp)                 # 8-byte Spill
	movq	%rdx, %rsp
	movq	%rax, -16(%rcx)
	movq	%rsp, %rax
	movq	%rax, %rdx
	addq	$-16, %rdx
	movq	%rdx, %rsp
	movq	-16(%rcx), %rcx
	movq	%rcx, -16(%rax)
	movq	-16(%rax), %rdi
	callq	write@PLT
	xorl	%eax, %eax
	movl	%eax, %edi
	callq	alloc_int@PLT
	movq	%rax, %rdx
	movq	-32(%rbp), %rax                 # 8-byte Reload
	movq	%rsp, %rcx
	addq	$-16, %rcx
	movq	%rcx, -24(%rbp)                 # 8-byte Spill
	movq	%rcx, %rsp
	movq	%rdx, (%rcx)
	movq	(%rax), %rdi
	callq	del_obj@PLT
	movq	-32(%rbp), %rcx                 # 8-byte Reload
	movq	-24(%rbp), %rax                 # 8-byte Reload
	movq	$0, (%rcx)
	movq	(%rax), %rdi
	movq	%rdi, -16(%rbp)                 # 8-byte Spill
	callq	inc_ref@PLT
	movq	-16(%rbp), %rdi                 # 8-byte Reload
	callq	get_int_value@PLT
	movq	-16(%rbp), %rdi                 # 8-byte Reload
	movl	%eax, -4(%rbp)                  # 4-byte Spill
	callq	del_obj@PLT
	movl	-4(%rbp), %eax                  # 4-byte Reload
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end1:
	.size	main, .Lfunc_end1-main
	.cfi_endproc
                                        # -- End function
	.section	".note.GNU-stack","",@progbits
