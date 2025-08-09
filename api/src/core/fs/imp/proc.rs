use crate::core::fs::dynamic::dynamic::{DirMaker, DynamicDir, DynamicFs};
use crate::core::fs::dynamic::file::SimpleFile;
use alloc::format;
use alloc::string::ToString;
use alloc::sync::Arc;
use axsync::RawMutex;
use undefined_vfs::fs::Filesystem;

const PID_MAX: i32 = 4194304;
const SHMMAX: i32 = 134217728;
const SHMMNI: i32 = 4096;
const STAT: &str = "1 (systemd) S 0 1 1 0 -1 4194304 1234 0 0 0 12 34 0 0 0 0 1 0 123456 12345678 456 18446744073709551615 0x400000 0x401000 0x7fff12345678 0x7fff12345000 0x400123 0 0 0x00000000 0x00000000 0 0 0 17 0 0 0 0 0 0x600000 0x601000 0x602000 0x7fff12346000 0x7fff12346100 0x7fff12346100 0x7fff12346200 0";
const EMPTY: &str = "100";
const CORE_PATTERN: &str = "|/wsl-capture-crash %t %E %p %s";
const PIPE_MAX_SIZE: &str = "1048576";
const LEASE_BREAK_TIME: &str = "45";
const CPUINFO: &str = "processor       : 0
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 0
cpu cores       : 8
apicid          : 0
initial apicid  : 0
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 1
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 0
cpu cores       : 8
apicid          : 1
initial apicid  : 1
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 2
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 1
cpu cores       : 8
apicid          : 2
initial apicid  : 2
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 3
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 1
cpu cores       : 8
apicid          : 3
initial apicid  : 3
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 4
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 2
cpu cores       : 8
apicid          : 4
initial apicid  : 4
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 5
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 2
cpu cores       : 8
apicid          : 5
initial apicid  : 5
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 6
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 3
cpu cores       : 8
apicid          : 6
initial apicid  : 6
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 7
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 3
cpu cores       : 8
apicid          : 7
initial apicid  : 7
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 8
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 4
cpu cores       : 8
apicid          : 8
initial apicid  : 8
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 9
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 4
cpu cores       : 8
apicid          : 9
initial apicid  : 9
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 10
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 5
cpu cores       : 8
apicid          : 10
initial apicid  : 10
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 11
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 5
cpu cores       : 8
apicid          : 11
initial apicid  : 11
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 12
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 6
cpu cores       : 8
apicid          : 12
initial apicid  : 12
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 13
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 6
cpu cores       : 8
apicid          : 13
initial apicid  : 13
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 14
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 7
cpu cores       : 8
apicid          : 14
initial apicid  : 14
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:

processor       : 15
vendor_id       : AuthenticAMD
cpu family      : 25
model           : 116
model name      : AMD Ryzen 7 7840HS w/ Radeon 780M Graphics
stepping        : 1
microcode       : 0xffffffff
cpu MHz         : 3792.478
cache size      : 1024 KB
physical id     : 0
siblings        : 16
core id         : 7
cpu cores       : 8
apicid          : 15
initial apicid  : 15
fpu             : yes
fpu_exception   : yes
cpuid level     : 13
wp              : yes
flags           : fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid avx512f avx512dq rdseed adx smap avx512ifma clflushopt clwb avx512cd sha_ni avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves avx512_bf16 clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold v_vmsave_vmload avx512vbmi umip avx512_vbmi2 gfni vaes vpclmulqdq avx512_vnni avx512_bitalg avx512_vpopcntdq rdpid
bugs            : sysret_ss_attrs null_seg spectre_v1 spectre_v2 spec_store_bypass srso
bogomips        : 7584.95
TLB size        : 2560 4K pages
clflush size    : 64
cache_alignment : 64
address sizes   : 48 bits physical, 48 bits virtual
power management:";
const MAP : &str = "
58b273d66000-58b273d96000 r--p 00000000 08:30 1462                       /usr/bin/bash
58b273d96000-58b273e85000 r-xp 00030000 08:30 1462                       /usr/bin/bash
58b273e85000-58b273eba000 r--p 0011f000 08:30 1462                       /usr/bin/bash
58b273eba000-58b273ebe000 r--p 00154000 08:30 1462                       /usr/bin/bash
58b273ebe000-58b273ec7000 rw-p 00158000 08:30 1462                       /usr/bin/bash
58b273ec7000-58b273ed2000 rw-p 00000000 00:00 0
58b2b344d000-58b2b35d9000 rw-p 00000000 00:00 0                          [heap]
7c3c899a4000-7c3c899fd000 r--p 00000000 08:30 1797                       /usr/lib/locale/C.utf8/LC_CTYPE
7c3c899fd000-7c3c899fe000 r--p 00000000 08:30 1837                       /usr/lib/locale/C.utf8/LC_NUMERIC
7c3c899fe000-7c3c899ff000 r--p 00000000 08:30 1877                       /usr/lib/locale/C.utf8/LC_TIME
7c3c899ff000-7c3c89a00000 r--p 00000000 08:30 1796                       /usr/lib/locale/C.utf8/LC_COLLATE
7c3c89a00000-7c3c89a28000 r--p 00000000 08:30 49956                      /usr/lib/x86_64-linux-gnu/libc.so.6
7c3c89a28000-7c3c89bb0000 r-xp 00028000 08:30 49956                      /usr/lib/x86_64-linux-gnu/libc.so.6
7c3c89bb0000-7c3c89bff000 r--p 001b0000 08:30 49956                      /usr/lib/x86_64-linux-gnu/libc.so.6
7c3c89bff000-7c3c89c03000 r--p 001fe000 08:30 49956                      /usr/lib/x86_64-linux-gnu/libc.so.6
7c3c89c03000-7c3c89c05000 rw-p 00202000 08:30 49956                      /usr/lib/x86_64-linux-gnu/libc.so.6
7c3c89c05000-7c3c89c12000 rw-p 00000000 00:00 0
7c3c89c12000-7c3c89c13000 r--p 00000000 08:30 1833                       /usr/lib/locale/C.utf8/LC_MONETARY
7c3c89c13000-7c3c89c14000 r--p 00000000 08:30 1825                       /usr/lib/locale/C.utf8/LC_MESSAGES/SYS_LC_MESSAGES
7c3c89c14000-7c3c89c15000 r--p 00000000 08:30 1855                       /usr/lib/locale/C.utf8/LC_PAPER
7c3c89c15000-7c3c89c18000 rw-p 00000000 00:00 0
7c3c89c18000-7c3c89c26000 r--p 00000000 08:30 13012                      /usr/lib/x86_64-linux-gnu/libtinfo.so.6.4
7c3c89c26000-7c3c89c39000 r-xp 0000e000 08:30 13012                      /usr/lib/x86_64-linux-gnu/libtinfo.so.6.4
7c3c89c39000-7c3c89c47000 r--p 00021000 08:30 13012                      /usr/lib/x86_64-linux-gnu/libtinfo.so.6.4
7c3c89c47000-7c3c89c4b000 r--p 0002e000 08:30 13012                      /usr/lib/x86_64-linux-gnu/libtinfo.so.6.4
7c3c89c4b000-7c3c89c4c000 rw-p 00032000 08:30 13012                      /usr/lib/x86_64-linux-gnu/libtinfo.so.6.4
7c3c89c4c000-7c3c89c4d000 r--p 00000000 08:30 1835                       /usr/lib/locale/C.utf8/LC_NAME
7c3c89c4d000-7c3c89c4e000 r--p 00000000 08:30 1795                       /usr/lib/locale/C.utf8/LC_ADDRESS
7c3c89c4e000-7c3c89c4f000 r--p 00000000 08:30 1865                       /usr/lib/locale/C.utf8/LC_TELEPHONE
7c3c89c4f000-7c3c89c50000 r--p 00000000 08:30 1821                       /usr/lib/locale/C.utf8/LC_MEASUREMENT
7c3c89c50000-7c3c89c57000 r--s 00000000 08:30 49945                      /usr/lib/x86_64-linux-gnu/gconv/gconv-modules.cache
7c3c89c57000-7c3c89c58000 r--p 00000000 08:30 1799                       /usr/lib/locale/C.utf8/LC_IDENTIFICATION
7c3c89c58000-7c3c89c5a000 rw-p 00000000 00:00 0
7c3c89c5a000-7c3c89c5b000 r--p 00000000 08:30 49953                      /usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2
7c3c89c5b000-7c3c89c86000 r-xp 00001000 08:30 49953                      /usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2
7c3c89c86000-7c3c89c90000 r--p 0002c000 08:30 49953                      /usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2
7c3c89c90000-7c3c89c92000 r--p 00036000 08:30 49953                      /usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2
7c3c89c92000-7c3c89c94000 rw-p 00038000 08:30 49953                      /usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2
7ffc074ac000-7ffc074cd000 rw-p 00000000 00:00 0                          [stack]
7ffc075ce000-7ffc075d2000 r--p 00000000 00:00 0                          [vvar]
7ffc075d2000-7ffc075d4000 r-xp 00000000 00:00 0                          [vdso]";

const MAPS: &str = "
00400000-0040b000 r-xp 00000000 08:01 1234567 /bin/bash
0060a000-0060b000 r--p 0000a000 08:01 1234567 /bin/bash
0060b000-0060c000 rw-p 0000b000 08:01 1234567 /bin/bash
0060c000-00611000 rw-p 00000000 00:00 0       [heap]
7f8b12345000-7f8b12367000 r-xp 00000000 08:01 2345678 /lib/x86_64-linux-gnu/libc-2.31.so
7f8b12367000-7f8b12567000 ---p 00022000 08:01 2345678 /lib/x86_64-linux-gnu/libc-2.31.so
7f8b12567000-7f8b1256b000 r--p 00022000 08:01 2345678 /lib/x86_64-linux-gnu/libc-2.31.so
7f8b1256b000-7f8b1256d000 rw-p 00026000 08:01 2345678 /lib/x86_64-linux-gnu/libc-2.31.so
7f8b1256d000-7f8b12571000 rw-p 00000000 00:00 0
7f8b12571000-7f8b1258a000 r-xp 00000000 08:01 2345679 /lib/x86_64-linux-gnu/ld-2.31.so
7f8b12789000-7f8b1278a000 r--p 00018000 08:01 2345679 /lib/x86_64-linux-gnu/ld-2.31.so
7f8b1278a000-7f8b1278b000 rw-p 00019000 08:01 2345679 /lib/x86_64-linux-gnu/ld-2.31.so
7ffc1c7e4000-7ffc1c805000 rw-p 00000000 00:00 0       [stack]
7ffc1c81f000-7ffc1c822000 r--p 00000000 00:00 0       [vvar]
7ffc1c822000-7ffc1c823000 r-xp 00000000 00:00 0       [vdso]
ffffffffff600000-ffffffffff601000 r-xp 00000000 00:00 0 [vsyscall]";
const DUMMY_MEMINFO: &str = "MemTotal:       32536204 kB
MemFree:         5506524 kB
MemAvailable:   18768344 kB
Buffers:            3264 kB
Cached:         14454588 kB
SwapCached:            0 kB
Active:         18229700 kB
Inactive:        6540624 kB
Active(anon):   11380224 kB
Inactive(anon):        0 kB
Active(file):    6849476 kB
Inactive(file):  6540624 kB
Unevictable:      930088 kB
Mlocked:            1136 kB
SwapTotal:       4194300 kB
SwapFree:        4194300 kB
Zswap:                 0 kB
Zswapped:              0 kB
Dirty:             47952 kB
Writeback:             0 kB
AnonPages:      10992512 kB
Mapped:          1361184 kB
Shmem:           1068056 kB
KReclaimable:     341440 kB
Slab:             628996 kB
SReclaimable:     341440 kB
SUnreclaim:       287556 kB
KernelStack:       28704 kB
PageTables:        85308 kB
SecPageTables:      2084 kB
NFS_Unstable:          0 kB
Bounce:                0 kB
WritebackTmp:          0 kB
CommitLimit:    20462400 kB
Committed_AS:   45105316 kB
VmallocTotal:   34359738367 kB
VmallocUsed:      205924 kB
VmallocChunk:          0 kB
Percpu:            23840 kB
HardwareCorrupted:     0 kB
AnonHugePages:   1417216 kB
ShmemHugePages:        0 kB
ShmemPmdMapped:        0 kB
FileHugePages:    477184 kB
FilePmdMapped:    288768 kB
CmaTotal:              0 kB
CmaFree:               0 kB
Unaccepted:            0 kB
HugePages_Total:       0
HugePages_Free:        0
HugePages_Rsvd:        0
HugePages_Surp:        0
Hugepagesize:       2048 kB
Hugetlb:               0 kB
DirectMap4k:     1739900 kB
DirectMap2M:    31492096 kB
DirectMap1G:     1048576 kB
";
const PRINTK: &str = "4       4       1       7";
const STATUS: &str = "Name:   bash
Umask:  0022
State:  S (sleeping)
Tgid:   44452
Ngid:   0
Pid:    44452
PPid:   44451
TracerPid:      0
Uid:       0       0       0       0
Gid:    1000    1000    1000    1000
FDSize: 256
Groups: 4 20 24 25 27 29 30 44 46 100 107 1000 1001
NStgid: 44452
NSpid:  44452
NSpgid: 44452
NSsid:  44452
Kthread:        0
VmPeak:     6204 kB
VmSize:     6204 kB
VmLck:         0 kB
VmPin:         0 kB
VmHWM:      5248 kB
VmRSS:      5248 kB
RssAnon:            1664 kB
RssFile:            3584 kB
RssShmem:              0 kB
VmData:     1756 kB
VmStk:       132 kB
VmExe:       956 kB
VmLib:      1824 kB
VmPTE:        52 kB
VmSwap:        0 kB
HugetlbPages:          0 kB
CoreDumping:    0
THP_enabled:    1
untag_mask:     0xffffffffffffffff
Threads:        1
SigQ:   1/30057
SigPnd: 0000000000000000
ShdPnd: 0000000000000000
SigBlk: 0000000000010000
SigIgn: 0000000000384004
SigCgt: 000000004b813efb
CapInh: 0000000000000000
CapPrm: 0000000000000000
CapEff: 0000000000000000
CapBnd: 000001ffffffffff
CapAmb: 0000000000000000
NoNewPrivs:     0
Seccomp:        2
Seccomp_filters:        1
Speculation_Store_Bypass:       thread vulnerable
SpeculationIndirectBranch:      conditional enabled
Cpus_allowed:   ffff
Cpus_allowed_list:      0-15
Mems_allowed:   00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000001
Mems_allowed_list:      0
voluntary_ctxt_switches:        107
nonvoluntary_ctxt_switches:     1";
pub fn new_procfs() -> Filesystem<RawMutex> {
    DynamicFs::new_with("proc".into(), 0x9fa0, builder)
}

fn builder(fs: Arc<DynamicFs>) -> DirMaker {
    let mut root = DynamicDir::builder(fs.clone());
    // '/proc/sys/kernel'
    let mut kernel = DynamicDir::builder(fs.clone());
    let mut selfs = DynamicDir::builder(fs.clone());
    let mut FS = DynamicDir::builder(fs.clone());
    FS.add(
        "pipe-max-size",
        SimpleFile::new(fs.clone(), || PIPE_MAX_SIZE.to_string()),
    );
    FS.add(
        "lease-break-time",
        SimpleFile::new(fs.clone(), || LEASE_BREAK_TIME.to_string()),
    );
    selfs.add("maps", SimpleFile::new(fs.clone(), || MAPS));
    selfs.add("status", SimpleFile::new(fs.clone(), || STATUS));
    selfs.add("stat", SimpleFile::new(fs.clone(), || STAT));
    kernel.add(
        "pid_max",
        SimpleFile::new(fs.clone(), || PID_MAX.to_string()),
    );
    kernel.add("shmmax", SimpleFile::new(fs.clone(), || SHMMAX.to_string()));
    kernel.add("shmmni", SimpleFile::new(fs.clone(), || SHMMNI.to_string()));
    kernel.add("printk", SimpleFile::new(fs.clone(), || PRINTK.to_string()));
    kernel.add(
        "core_pattern",
        SimpleFile::new(fs.clone(), || CORE_PATTERN.to_string()),
    );
    // '/proc/sys'
    let mut sys = DynamicDir::builder(fs.clone());
    sys.add("kernel", kernel.build());
    sys.add("fs", FS.build());
    root.add("sys", sys.build());
    let mut one = DynamicDir::builder(fs.clone());
    one.add("stat", SimpleFile::new(fs.clone(), || STAT));
    let mut ten = DynamicDir::builder(fs.clone());
    ten.add("stat", SimpleFile::new(fs.clone(), || STAT));
    root.add("1", one.build());
    root.add("10", ten.build());
    let mut sysvipc = DynamicDir::builder(fs.clone());
    sysvipc.add("shm", SimpleFile::new(fs.clone(), || EMPTY));
    root.add("sysvipc", sysvipc.build());
    root.add(
        "mounts",
        SimpleFile::new(
            fs.clone(),
            || "proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0\n",
        ),
    );
    root.add("meminfo", SimpleFile::new(fs.clone(), || DUMMY_MEMINFO));
    root.add("cpuinfo", SimpleFile::new(fs.clone(), || CPUINFO));
    root.add("self", selfs.build());

    // '/proc/interrupts'
    root.add(
        "interrupts",
        SimpleFile::new(fs.clone(), || format!("0: {}", axtask::get_irq_count())),
    );

    root.build()
}
