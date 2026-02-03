	.text
	.file	"ayanami_modlue"
	.globl	partition                       # -- Begin function partition
	.p2align	4, 0x90
	.type	partition,@function
partition:                              # @partition
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
	subq	$168, %rsp
	.cfi_offset %rbx, -56
	.cfi_offset %r12, -48
	.cfi_offset %r13, -40
	.cfi_offset %r14, -32
	.cfi_offset %r15, -24
	movq	%rdi, -48(%rbp)
	movq	%rsi, -168(%rbp)
	movq	%rdx, -72(%rbp)
	movq	%rsi, -144(%rbp)
	movq	%rsi, %rdi
	callq	inc_ref@PLT
	movq	-144(%rbp), %rax
	movq	%rax, -56(%rbp)
	movq	-72(%rbp), %rdi
	movq	%rdi, -200(%rbp)
	movq	-48(%rbp), %rax
	movq	%rax, -208(%rbp)
	movq	16(%rax), %rbx
	callq	get_int_value@PLT
	cltq
	movq	(%rbx,%rax,8), %rdi
	movq	%rdi, -152(%rbp)
	callq	inc_ref@PLT
	movq	-152(%rbp), %rax
	movq	%rax, -176(%rbp)
	movq	-168(%rbp), %rdi
	movq	%rdi, -160(%rbp)
	callq	inc_ref@PLT
	movq	-160(%rbp), %rax
	jmp	.LBB0_1
	.p2align	4, 0x90
.LBB0_4:                                # %block_merge_7
                                        #   in Loop: Header=BB0_1 Depth=1
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %r15
	leaq	-16(%r15), %rsp
	movq	-64(%rbp), %rax
	movq	%rax, -16(%r15)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -16(%r14)
	movq	-16(%r15), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	%rax, -16(%rbx)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-16(%rbx), %rax
.LBB0_1:                                # %block_2
                                        # =>This Inner Loop Header: Depth=1
	movq	%rax, -64(%rbp)
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %rax
	leaq	-16(%rax), %rsp
	movq	%rsp, %rcx
	leaq	-16(%rcx), %rsp
	movq	-64(%rbp), %rdx
	movq	%rdx, -16(%rcx)
	movq	-72(%rbp), %rsi
	movq	%rsi, -16(%rax)
	movq	-16(%rcx), %rdi
	callq	less@PLT
	movq	%rax, -16(%rbx)
	movq	%rax, %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB0_5
# %bb.2:                                # %block_3
                                        #   in Loop: Header=BB0_1 Depth=1
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %r15
	leaq	-16(%r15), %rsp
	movq	%rsp, %rax
	leaq	-16(%rax), %rsp
	movq	%rsp, %rcx
	leaq	-16(%rcx), %rsp
	movq	-64(%rbp), %rdx
	movq	%rdx, -16(%rcx)
	movq	-48(%rbp), %rdx
	movq	%rdx, -16(%rax)
	movq	16(%rdx), %r12
	movq	-16(%rcx), %rdi
	callq	get_int_value@PLT
	cltq
	movq	(%r12,%rax,8), %rax
	movq	%rax, -16(%r15)
	movq	-176(%rbp), %rsi
	movq	%rsi, -16(%r14)
	movq	-16(%r15), %rdi
	callq	less@PLT
	movq	%rax, -16(%rbx)
	movq	%rax, %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB0_4
# %bb.3:                                # %block_6
                                        #   in Loop: Header=BB0_1 Depth=1
	movq	%rsp, %rax
	movq	%rax, -120(%rbp)                # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -112(%rbp)                # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -104(%rbp)                # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -96(%rbp)                 # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -88(%rbp)                 # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -80(%rbp)                 # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -136(%rbp)                # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -128(%rbp)                # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %r13
	leaq	-16(%r13), %rsp
	movq	%rsp, %rax
	leaq	-16(%rax), %rsp
	movq	%rsp, %rcx
	leaq	-16(%rcx), %rsp
	movq	-56(%rbp), %rdx
	movq	%rdx, -16(%rcx)
	movq	-48(%rbp), %rdx
	movq	%rdx, -16(%rax)
	movq	16(%rdx), %r15
	movq	-16(%rcx), %rdi
	callq	get_int_value@PLT
	cltq
	movq	(%r15,%rax,8), %rdi
	movq	%rdi, -16(%r13)
	callq	inc_ref@PLT
	movq	-16(%r13), %rax
	movq	%rax, -184(%rbp)
	movq	-64(%rbp), %rax
	movq	%rax, -16(%r14)
	movq	-48(%rbp), %rax
	movq	%rax, -16(%r12)
	movq	16(%rax), %r15
	movq	-16(%r14), %rdi
	callq	get_int_value@PLT
	cltq
	movq	(%r15,%rax,8), %rax
	movq	%rax, -16(%rbx)
	movq	-56(%rbp), %rax
	movq	-128(%rbp), %r15                # 8-byte Reload
	movq	%rax, -16(%r15)
	movq	-16(%rbx), %rdi
	callq	inc_ref@PLT
	movq	-48(%rbp), %rax
	movq	-136(%rbp), %rcx                # 8-byte Reload
	movq	%rax, -16(%rcx)
	movq	16(%rax), %r14
	movq	-16(%r15), %rdi
	callq	get_int_value@PLT
	cltq
	movq	-16(%rbx), %rcx
	movq	%rcx, (%r14,%rax,8)
	movq	-184(%rbp), %rax
	movq	-80(%rbp), %rcx                 # 8-byte Reload
	movq	%rax, -16(%rcx)
	movq	-64(%rbp), %rax
	movq	-88(%rbp), %r15                 # 8-byte Reload
	movq	%rax, -16(%r15)
	movq	-16(%rcx), %rdi
	movq	%rcx, %r14
	callq	inc_ref@PLT
	movq	-48(%rbp), %rax
	movq	-96(%rbp), %rcx                 # 8-byte Reload
	movq	%rax, -16(%rcx)
	movq	16(%rax), %rbx
	movq	-16(%r15), %rdi
	callq	get_int_value@PLT
	cltq
	movq	-16(%r14), %rcx
	movq	%rcx, (%rbx,%rax,8)
	movq	-56(%rbp), %rax
	movq	-104(%rbp), %rbx                # 8-byte Reload
	movq	%rax, -16(%rbx)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	-112(%rbp), %rcx                # 8-byte Reload
	movq	%rax, -16(%rcx)
	movq	-16(%rbx), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	-120(%rbp), %rbx                # 8-byte Reload
	movq	%rax, -16(%rbx)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-16(%rbx), %rax
	movq	%rax, -56(%rbp)
	jmp	.LBB0_4
.LBB0_5:                                # %block_merge_4
	movq	%rsp, %rax
	movq	%rax, -120(%rbp)                # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -112(%rbp)                # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -104(%rbp)                # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -96(%rbp)                 # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -88(%rbp)                 # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %rax
	movq	%rax, -80(%rbp)                 # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %r13
	leaq	-16(%r13), %rsp
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	%rsp, %r15
	leaq	-16(%r15), %rsp
	movq	%rsp, %rax
	leaq	-16(%rax), %rsp
	movq	%rsp, %rcx
	leaq	-16(%rcx), %rsp
	movq	-56(%rbp), %rdx
	movq	%rdx, -16(%rcx)
	movq	-48(%rbp), %rdx
	movq	%rdx, -16(%rax)
	movq	16(%rdx), %rbx
	movq	-16(%rcx), %rdi
	callq	get_int_value@PLT
	cltq
	movq	(%rbx,%rax,8), %rdi
	movq	%rdi, -16(%r15)
	callq	inc_ref@PLT
	movq	-16(%r15), %rax
	movq	%rax, -192(%rbp)
	movq	-72(%rbp), %rax
	movq	%rax, -16(%r12)
	movq	-48(%rbp), %rax
	movq	%rax, -16(%r13)
	movq	16(%rax), %rbx
	movq	-16(%r12), %rdi
	callq	get_int_value@PLT
	cltq
	movq	(%rbx,%rax,8), %rax
	movq	%rax, -16(%r14)
	movq	-56(%rbp), %rax
	movq	-80(%rbp), %r15                 # 8-byte Reload
	movq	%rax, -16(%r15)
	movq	-16(%r14), %rdi
	callq	inc_ref@PLT
	movq	-48(%rbp), %rax
	movq	-88(%rbp), %rcx                 # 8-byte Reload
	movq	%rax, -16(%rcx)
	movq	16(%rax), %rbx
	movq	-16(%r15), %rdi
	callq	get_int_value@PLT
	cltq
	movq	-16(%r14), %rcx
	movq	%rcx, (%rbx,%rax,8)
	movq	-192(%rbp), %rax
	movq	-96(%rbp), %rcx                 # 8-byte Reload
	movq	%rax, -16(%rcx)
	movq	-72(%rbp), %rax
	movq	-104(%rbp), %r15                # 8-byte Reload
	movq	%rax, -16(%r15)
	movq	-16(%rcx), %rdi
	movq	%rcx, %r14
	callq	inc_ref@PLT
	movq	-48(%rbp), %rax
	movq	-112(%rbp), %rcx                # 8-byte Reload
	movq	%rax, -16(%rcx)
	movq	16(%rax), %rbx
	movq	-16(%r15), %rdi
	callq	get_int_value@PLT
	cltq
	movq	-16(%r14), %rcx
	movq	%rcx, (%rbx,%rax,8)
	movq	-56(%rbp), %rbx
	movq	-120(%rbp), %rax                # 8-byte Reload
	movq	%rbx, -16(%rax)
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
	.size	partition, .Lfunc_end0-partition
	.cfi_endproc
                                        # -- End function
	.globl	qsort                           # -- Begin function qsort
	.p2align	4, 0x90
	.type	qsort,@function
qsort:                                  # @qsort
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
	movq	%rdi, -48(%rbp)
	movq	%rsi, -80(%rbp)
	movq	%rdx, -72(%rbp)
	movq	%rsi, -88(%rbp)
	movq	%rdx, -96(%rbp)
	movq	%rsi, %rdi
	movq	%rdx, %rsi
	callq	less@PLT
	movq	%rax, -104(%rbp)
	movq	%rax, %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB1_4
# %bb.1:                                # %block_11
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %r15
	leaq	-16(%r15), %rsp
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	-48(%rbp), %rdi
	movq	%rdi, -16(%r12)
	callq	inc_ref@PLT
	movq	-80(%rbp), %rdi
	movq	%rdi, -16(%r15)
	callq	inc_ref@PLT
	movq	-72(%rbp), %rdi
	movq	%rdi, -16(%r14)
	callq	inc_ref@PLT
	movq	-16(%r12), %rdi
	movq	-16(%r15), %rsi
	movq	-16(%r14), %rdx
	callq	partition@PLT
	movq	%rax, -16(%rbx)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-16(%rbx), %rax
	movq	%rax, -64(%rbp)
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %r15
	leaq	-16(%r15), %rsp
	movq	-64(%rbp), %rax
	movq	%rax, -16(%r15)
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, -16(%r14)
	movq	-16(%r15), %rdi
	movq	%rax, %rsi
	callq	greater@PLT
	movq	%rax, -16(%rbx)
	movq	%rax, %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB1_3
# %bb.2:                                # %block_14
	movq	%rsp, %rax
	movq	%rax, -56(%rbp)                 # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %r15
	leaq	-16(%r15), %rsp
	movq	%rsp, %r13
	leaq	-16(%r13), %rsp
	movq	-48(%rbp), %rdi
	movq	%rdi, -16(%r13)
	callq	inc_ref@PLT
	movq	-80(%rbp), %rdi
	movq	%rdi, -16(%r15)
	callq	inc_ref@PLT
	movq	-64(%rbp), %rax
	movq	%rax, -16(%rbx)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -16(%r12)
	movq	-16(%rbx), %rdi
	movq	%rax, %rsi
	callq	sub@PLT
	movq	%rax, -16(%r14)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-16(%r13), %rdi
	movq	-16(%r15), %rsi
	movq	-16(%r14), %rdx
	callq	qsort@PLT
	movq	-56(%rbp), %rcx                 # 8-byte Reload
	movq	%rax, -16(%rcx)
.LBB1_3:                                # %block_merge_15
	movq	%rsp, %rax
	movq	%rax, -56(%rbp)                 # 8-byte Spill
	leaq	-16(%rax), %rsp
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %r15
	leaq	-16(%r15), %rsp
	movq	%rsp, %r13
	leaq	-16(%r13), %rsp
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	-48(%rbp), %rdi
	movq	%rdi, -16(%r12)
	callq	inc_ref@PLT
	movq	-64(%rbp), %rax
	movq	%rax, -16(%rbx)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -16(%r13)
	movq	-16(%rbx), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	%rax, -16(%r15)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-72(%rbp), %rdi
	movq	%rdi, -16(%r14)
	callq	inc_ref@PLT
	movq	-16(%r12), %rdi
	movq	-16(%r15), %rsi
	movq	-16(%r14), %rdx
	callq	qsort@PLT
	movq	-56(%rbp), %rcx                 # 8-byte Reload
	movq	%rax, -16(%rcx)
.LBB1_4:                                # %block_merge_12
	movq	%rsp, %rax
	leaq	-16(%rax), %rsp
	movq	-48(%rbp), %rbx
	movq	%rbx, -16(%rax)
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
.Lfunc_end1:
	.size	qsort, .Lfunc_end1-qsort
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
	subq	$152, %rsp
	.cfi_offset %rbx, -56
	.cfi_offset %r12, -48
	.cfi_offset %r13, -40
	.cfi_offset %r14, -32
	.cfi_offset %r15, -24
	movl	$5, %edi
	callq	alloc_int@PLT
	movq	%rax, -80(%rbp)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -88(%rbp)
	movl	$4, %edi
	callq	alloc_int@PLT
	movq	%rax, -96(%rbp)
	movl	$2, %edi
	callq	alloc_int@PLT
	movq	%rax, -104(%rbp)
	movl	$8, %edi
	callq	alloc_int@PLT
	movq	%rax, -112(%rbp)
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, -120(%rbp)
	movl	$3, %edi
	callq	alloc_int@PLT
	movq	%rax, -128(%rbp)
	movl	$7, %edi
	callq	alloc_array@PLT
	movq	%rax, -56(%rbp)
	movl	$7, 8(%rax)
	movq	16(%rax), %rax
	movq	-80(%rbp), %rcx
	movq	%rcx, (%rax)
	movq	-88(%rbp), %rcx
	movq	%rcx, 8(%rax)
	movq	-96(%rbp), %rcx
	movq	%rcx, 16(%rax)
	movq	-104(%rbp), %rcx
	movq	%rcx, 24(%rax)
	movq	-112(%rbp), %rcx
	movq	%rcx, 32(%rax)
	movq	-120(%rbp), %rcx
	movq	%rcx, 40(%rax)
	movq	-128(%rbp), %rcx
	movq	%rcx, 48(%rax)
	movq	-56(%rbp), %rdi
	callq	inc_ref@PLT
	movq	-56(%rbp), %rax
	movq	%rax, -64(%rbp)
	movl	$7, %edi
	callq	alloc_int@PLT
	movq	%rax, -136(%rbp)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-136(%rbp), %rax
	movq	%rax, -72(%rbp)
	movq	-64(%rbp), %rdi
	movq	%rdi, -144(%rbp)
	callq	inc_ref@PLT
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, -152(%rbp)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-72(%rbp), %rax
	movq	%rax, -160(%rbp)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -184(%rbp)
	movq	-160(%rbp), %rdi
	movq	%rax, %rsi
	callq	sub@PLT
	movq	%rax, -168(%rbp)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-144(%rbp), %rdi
	movq	-152(%rbp), %rsi
	movq	-168(%rbp), %rdx
	callq	qsort@PLT
	movq	%rax, -192(%rbp)
	xorl	%edi, %edi
	callq	alloc_int@PLT
	movq	%rax, -176(%rbp)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-176(%rbp), %rax
	.p2align	4, 0x90
.LBB2_1:                                # %block_18
                                        # =>This Inner Loop Header: Depth=1
	movq	%rax, -48(%rbp)
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %rax
	leaq	-16(%rax), %rsp
	movq	%rsp, %rcx
	leaq	-16(%rcx), %rsp
	movq	-48(%rbp), %rdx
	movq	%rdx, -16(%rcx)
	movq	-72(%rbp), %rsi
	movq	%rsi, -16(%rax)
	movq	-16(%rcx), %rdi
	callq	less@PLT
	movq	%rax, -16(%rbx)
	movq	%rax, %rdi
	callq	is_truth@PLT
	testb	$1, %al
	je	.LBB2_3
# %bb.2:                                # %block_19
                                        #   in Loop: Header=BB2_1 Depth=1
	movq	%rsp, %rbx
	leaq	-16(%rbx), %rsp
	movq	%rsp, %r14
	leaq	-16(%r14), %rsp
	movq	%rsp, %r15
	leaq	-16(%r15), %rsp
	movq	%rsp, %r12
	leaq	-16(%r12), %rsp
	movq	%rsp, %rax
	leaq	-16(%rax), %rsp
	movq	%rsp, %rcx
	leaq	-16(%rcx), %rsp
	movq	-48(%rbp), %rdx
	movq	%rdx, -16(%rcx)
	movq	-64(%rbp), %rdx
	movq	%rdx, -16(%rax)
	movq	16(%rdx), %r13
	movq	-16(%rcx), %rdi
	callq	get_int_value@PLT
	cltq
	movq	(%r13,%rax,8), %rdi
	movq	%rdi, -16(%r12)
	callq	inc_ref@PLT
	movq	-16(%r12), %rdi
	callq	write@PLT
	movq	-48(%rbp), %rax
	movq	%rax, -16(%r15)
	movl	$1, %edi
	callq	alloc_int@PLT
	movq	%rax, -16(%r14)
	movq	-16(%r15), %rdi
	movq	%rax, %rsi
	callq	add@PLT
	movq	%rax, -16(%rbx)
	movq	%rax, %rdi
	callq	inc_ref@PLT
	movq	-16(%rbx), %rax
	jmp	.LBB2_1
.LBB2_3:                                # %block_merge_20
.Lfunc_end2:
	.size	main, .Lfunc_end2-main
	.cfi_endproc
                                        # -- End function
	.section	".note.GNU-stack","",@progbits
