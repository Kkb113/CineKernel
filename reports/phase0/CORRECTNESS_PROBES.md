# Correctness probes

| Probe | Status | Evidence |
|---|---|---|
| A — Repeated render stability | PASS | native-2d/typography-layout/smoke decoded framemd5 SHA-256 values 6a33df14dfd78a320bee14ba40c7516116c7b00e7489929bf7e920c21613c8e2 / 6a33df14dfd78a320bee14ba40c7516116c7b00e7489929bf7e920c21613c8e2. |
| B — Sequential versus parallel media sampling | PASS | remotion:default=ok[15,25,35,45 vs 15,25,35,45]; remotion:1=ok[15,25,35,45 vs 15,25,35,45]; remotion:4=ok[15,25,35,45 vs 15,25,35,45]; hyperframes:default=ok[15,25,35,45 vs 15,25,35,45]; hyperframes:1=ok[15,25,35,45 vs 15,25,35,45]; hyperframes:4=ok[15,25,35,45 vs 15,25,35,45] |
| C — Random-access versus sequential native evaluation | PASS | native-2d=identical 220bb8829981/220bb8829981; native-wgpu=identical 719fad08d58d/719fad08d58d |
| D — Preview/snapshot versus final frame | PASS | remotion=MAE 0.723 (ok); hyperframes=MAE 1.933 (ok) |
| E — Audio presence | PASS | Decoded full-profile mono RMS tone=0.08995, silence windows=0.00000/0.00000; 24 audio-bearing outputs retained. |
| F — Audio seam and overlap | PASS | Encoded max adjacent-sample jumps at 2/3/5/6s=0.00737/0.00000/0.01100/0.00001; required <0.08 with silent gap RMS <0.01. |
| G — Render-time network isolation | PASS | Static audit found no HTTP(S) render dependencies in either baseline composition; OS-level blocked-network rerun remains desirable. |
| H — Final mux integrity | PASS | 121 retained output(s) passed ffprobe track/frame/duration verification. |
| I — Process failure and cleanup | PASS | terminated=true; tracked_pids=28320,23744,13288; surviving_pids=none; partial_files=11; success_record=false; next_run_exit=0. |
| J — Controlled slow consumer | PASS | Bounded queue capacity=4, observed max=4, produced=100, consumed=100. |
