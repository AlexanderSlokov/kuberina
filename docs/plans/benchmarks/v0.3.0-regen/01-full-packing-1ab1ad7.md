# Full packing — commit `1ab1ad7` — 2026-09-06

Raw console output, ANSI escapes stripped. Cited by
`docs/references/sessions/2026-09-06-solver-audit.md`.

```
$ make solver-irina
```

## stderr (loader, FFD seed, GA progress)

```
Loaded 620 nodes, 4 daemonsets, 2714 pods, 0 groups
Phase 1 (FFD): seed fitness = 1582568.1260
Datacenter-scale detected (2714 pods) — cranking GA to maximum
Gen    0/1000 | best=1582568.1260 | stale=1   | ░░░░░░░░░░░░░░░░░░░░ 0%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0630
  [Nodes] Active: 540 / 620
Gen   10/1000 | best=1582568.1260 | stale=11  | ░░░░░░░░░░░░░░░░░░░░ 1%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0630
  [Nodes] Active: 540 / 620
Gen   20/1000 | best=1582568.1260 | stale=21  | ░░░░░░░░░░░░░░░░░░░░ 2%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0630
  [Nodes] Active: 540 / 620
Gen   30/1000 | best=1582568.1260 | stale=31  | ░░░░░░░░░░░░░░░░░░░░ 3%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0630
  [Nodes] Active: 540 / 620
Gen   40/1000 | best=1582568.1260 | stale=41  | ░░░░░░░░░░░░░░░░░░░░ 4%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0630
  [Nodes] Active: 540 / 620
Gen   50/1000 | best=1582568.1260 | stale=51  | █░░░░░░░░░░░░░░░░░░░ 5%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0630
  [Nodes] Active: 540 / 620
Gen   60/1000 | best=1582565.1260 | stale=6   | █░░░░░░░░░░░░░░░░░░░ 6%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0630
  [Nodes] Active: 540 / 620
Gen   70/1000 | best=1582565.1260 | stale=16  | █░░░░░░░░░░░░░░░░░░░ 7%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0630
  [Nodes] Active: 540 / 620
Gen   80/1000 | best=1582565.1260 | stale=4   | █░░░░░░░░░░░░░░░░░░░ 8%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0630
  [Nodes] Active: 540 / 620
Gen   90/1000 | best=1582565.1258 | stale=4   | █░░░░░░░░░░░░░░░░░░░ 9%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0629
  [Nodes] Active: 540 / 620
Gen  100/1000 | best=1582565.1256 | stale=1   | ██░░░░░░░░░░░░░░░░░░ 10%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0628
  [Nodes] Active: 540 / 620
Gen  110/1000 | best=1582562.1261 | stale=6   | ██░░░░░░░░░░░░░░░░░░ 11%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0630
  [Nodes] Active: 540 / 620
Gen  120/1000 | best=1582559.1259 | stale=1   | ██░░░░░░░░░░░░░░░░░░ 12%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0629
  [Nodes] Active: 540 / 620
Gen  130/1000 | best=1582559.1259 | stale=11  | ██░░░░░░░░░░░░░░░░░░ 13%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0629
  [Nodes] Active: 540 / 620
Gen  140/1000 | best=1582559.1254 | stale=1   | ██░░░░░░░░░░░░░░░░░░ 14%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0627
  [Nodes] Active: 540 / 620
Gen  150/1000 | best=1582559.1254 | stale=11  | ███░░░░░░░░░░░░░░░░░ 15%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0627
  [Nodes] Active: 540 / 620
Gen  160/1000 | best=1582556.1257 | stale=2   | ███░░░░░░░░░░░░░░░░░ 16%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0628
  [Nodes] Active: 540 / 620
Gen  170/1000 | best=1582556.1257 | stale=12  | ███░░░░░░░░░░░░░░░░░ 17%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0628
  [Nodes] Active: 540 / 620
Gen  180/1000 | best=1582556.1257 | stale=22  | ███░░░░░░░░░░░░░░░░░ 18%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0628
  [Nodes] Active: 540 / 620
Gen  190/1000 | best=1582556.1257 | stale=32  | ███░░░░░░░░░░░░░░░░░ 19%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0628
  [Nodes] Active: 540 / 620
Gen  200/1000 | best=1582556.1257 | stale=42  | ████░░░░░░░░░░░░░░░░ 20%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0628
  [Nodes] Active: 540 / 620
Gen  210/1000 | best=1582556.1256 | stale=2   | ████░░░░░░░░░░░░░░░░ 21%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0628
  [Nodes] Active: 540 / 620
Gen  220/1000 | best=1582556.1251 | stale=8   | ████░░░░░░░░░░░░░░░░ 22%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0626
  [Nodes] Active: 540 / 620
Gen  230/1000 | best=1582556.1251 | stale=18  | ████░░░░░░░░░░░░░░░░ 23%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0626
  [Nodes] Active: 540 / 620
Gen  240/1000 | best=1582556.1247 | stale=6   | ████░░░░░░░░░░░░░░░░ 24%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0623
  [Nodes] Active: 540 / 620
Gen  250/1000 | best=1582556.1247 | stale=16  | █████░░░░░░░░░░░░░░░ 25%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0623
  [Nodes] Active: 540 / 620
Gen  260/1000 | best=1582556.1244 | stale=0   | █████░░░░░░░░░░░░░░░ 26%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0622
  [Nodes] Active: 540 / 620
Gen  270/1000 | best=1582556.1244 | stale=10  | █████░░░░░░░░░░░░░░░ 27%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0622
  [Nodes] Active: 540 / 620
Gen  280/1000 | best=1582556.1238 | stale=1   | █████░░░░░░░░░░░░░░░ 28%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0619
  [Nodes] Active: 540 / 620
Gen  290/1000 | best=1582556.1238 | stale=11  | █████░░░░░░░░░░░░░░░ 29%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0619
  [Nodes] Active: 540 / 620
Gen  300/1000 | best=1582556.1236 | stale=7   | ██████░░░░░░░░░░░░░░ 30%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0618
  [Nodes] Active: 540 / 620
Gen  310/1000 | best=1582556.1236 | stale=5   | ██████░░░░░░░░░░░░░░ 31%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0618
  [Nodes] Active: 540 / 620
Gen  320/1000 | best=1582556.1231 | stale=5   | ██████░░░░░░░░░░░░░░ 32%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0616
  [Nodes] Active: 540 / 620
Gen  330/1000 | best=1582556.1231 | stale=15  | ██████░░░░░░░░░░░░░░ 33%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0616
  [Nodes] Active: 540 / 620
Gen  340/1000 | best=1582556.1231 | stale=9   | ██████░░░░░░░░░░░░░░ 34%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0615
  [Nodes] Active: 540 / 620
Gen  350/1000 | best=1582556.1227 | stale=0   | ███████░░░░░░░░░░░░░ 35%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0613
  [Nodes] Active: 540 / 620
Gen  360/1000 | best=1582556.1223 | stale=9   | ███████░░░░░░░░░░░░░ 36%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0611
  [Nodes] Active: 540 / 620
Gen  370/1000 | best=1582556.1219 | stale=1   | ███████░░░░░░░░░░░░░ 37%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0609
  [Nodes] Active: 540 / 620
Gen  380/1000 | best=1582556.1213 | stale=0   | ███████░░░░░░░░░░░░░ 38%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0607
  [Nodes] Active: 540 / 620
Gen  390/1000 | best=1582556.1213 | stale=10  | ███████░░░░░░░░░░░░░ 39%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0607
  [Nodes] Active: 540 / 620
Gen  400/1000 | best=1582556.1210 | stale=1   | ████████░░░░░░░░░░░░ 40%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0605
  [Nodes] Active: 540 / 620
Gen  410/1000 | best=1582556.1207 | stale=0   | ████████░░░░░░░░░░░░ 41%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0604
  [Nodes] Active: 540 / 620
Gen  420/1000 | best=1582556.1204 | stale=2   | ████████░░░░░░░░░░░░ 42%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0602
  [Nodes] Active: 540 / 620
Gen  430/1000 | best=1582556.1201 | stale=0   | ████████░░░░░░░░░░░░ 43%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0601
  [Nodes] Active: 540 / 620
Gen  440/1000 | best=1582556.1199 | stale=3   | ████████░░░░░░░░░░░░ 44%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0600
  [Nodes] Active: 540 / 620
Gen  450/1000 | best=1582556.1195 | stale=2   | █████████░░░░░░░░░░░ 45%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0598
  [Nodes] Active: 540 / 620
Gen  460/1000 | best=1582556.1191 | stale=7   | █████████░░░░░░░░░░░ 46%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0595
  [Nodes] Active: 540 / 620
Gen  470/1000 | best=1582556.1188 | stale=7   | █████████░░░░░░░░░░░ 47%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0594
  [Nodes] Active: 540 / 620
Gen  480/1000 | best=1582556.1184 | stale=2   | █████████░░░░░░░░░░░ 48%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0592
  [Nodes] Active: 540 / 620
Gen  490/1000 | best=1582556.1182 | stale=4   | █████████░░░░░░░░░░░ 49%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0591
  [Nodes] Active: 540 / 620
Gen  500/1000 | best=1582556.1179 | stale=0   | ██████████░░░░░░░░░░ 50%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0589
  [Nodes] Active: 540 / 620
Gen  510/1000 | best=1582556.1176 | stale=1   | ██████████░░░░░░░░░░ 51%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0588
  [Nodes] Active: 540 / 620
Gen  520/1000 | best=1582553.1189 | stale=6   | ██████████░░░░░░░░░░ 52%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0594
  [Nodes] Active: 540 / 620
Gen  530/1000 | best=1582553.1189 | stale=16  | ██████████░░░░░░░░░░ 53%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0594
  [Nodes] Active: 540 / 620
Gen  540/1000 | best=1582553.1181 | stale=4   | ██████████░░░░░░░░░░ 54%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0591
  [Nodes] Active: 540 / 620
Gen  550/1000 | best=1582553.1174 | stale=5   | ███████████░░░░░░░░░ 55%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0587
  [Nodes] Active: 540 / 620
Gen  560/1000 | best=1582553.1171 | stale=0   | ███████████░░░░░░░░░ 56%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0586
  [Nodes] Active: 540 / 620
Gen  570/1000 | best=1582553.1171 | stale=10  | ███████████░░░░░░░░░ 57%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0586
  [Nodes] Active: 540 / 620
Gen  580/1000 | best=1582553.1170 | stale=7   | ███████████░░░░░░░░░ 58%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1577120.00 | Aff: 0 | Var: 0.0585
  [Nodes] Active: 540 / 620
Gen  590/1000 | best=1572736.6180 | stale=0   | ███████████░░░░░░░░░ 59%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0590
  [Nodes] Active: 539 / 620
Gen  600/1000 | best=1572736.6180 | stale=10  | ████████████░░░░░░░░ 60%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0590
  [Nodes] Active: 539 / 620
Gen  610/1000 | best=1572736.6180 | stale=20  | ████████████░░░░░░░░ 61%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0590
  [Nodes] Active: 539 / 620
Gen  620/1000 | best=1572736.6176 | stale=8   | ████████████░░░░░░░░ 62%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0588
  [Nodes] Active: 539 / 620
Gen  630/1000 | best=1572736.6173 | stale=1   | ████████████░░░░░░░░ 63%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0586
  [Nodes] Active: 539 / 620
Gen  640/1000 | best=1572736.6170 | stale=6   | ████████████░░░░░░░░ 64%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0585
  [Nodes] Active: 539 / 620
Gen  650/1000 | best=1572736.6167 | stale=2   | █████████████░░░░░░░ 65%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0583
  [Nodes] Active: 539 / 620
Gen  660/1000 | best=1572736.6162 | stale=2   | █████████████░░░░░░░ 66%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0581
  [Nodes] Active: 539 / 620
Gen  670/1000 | best=1572736.6161 | stale=2   | █████████████░░░░░░░ 67%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0580
  [Nodes] Active: 539 / 620
Gen  680/1000 | best=1572736.6159 | stale=0   | █████████████░░░░░░░ 68%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0580
  [Nodes] Active: 539 / 620
Gen  690/1000 | best=1572736.6155 | stale=4   | █████████████░░░░░░░ 69%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0578
  [Nodes] Active: 539 / 620
Gen  700/1000 | best=1572736.6150 | stale=5   | ██████████████░░░░░░ 70%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0575
  [Nodes] Active: 539 / 620
Gen  710/1000 | best=1572736.6147 | stale=3   | ██████████████░░░░░░ 71%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0574
  [Nodes] Active: 539 / 620
Gen  720/1000 | best=1572736.6146 | stale=3   | ██████████████░░░░░░ 72%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0573
  [Nodes] Active: 539 / 620
Gen  730/1000 | best=1572736.6140 | stale=2   | ██████████████░░░░░░ 73%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0570
  [Nodes] Active: 539 / 620
Gen  740/1000 | best=1572736.6138 | stale=9   | ██████████████░░░░░░ 74%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0569
  [Nodes] Active: 539 / 620
Gen  750/1000 | best=1572736.6135 | stale=3   | ███████████████░░░░░ 75%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0568
  [Nodes] Active: 539 / 620
Gen  760/1000 | best=1572736.6127 | stale=2   | ███████████████░░░░░ 76%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0563
  [Nodes] Active: 539 / 620
Gen  770/1000 | best=1572736.6126 | stale=4   | ███████████████░░░░░ 77%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0563
  [Nodes] Active: 539 / 620
Gen  780/1000 | best=1572736.6121 | stale=9   | ███████████████░░░░░ 78%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0561
  [Nodes] Active: 539 / 620
Gen  790/1000 | best=1572736.6121 | stale=19  | ███████████████░░░░░ 79%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0561
  [Nodes] Active: 539 / 620
Gen  800/1000 | best=1572736.6113 | stale=1   | ████████████████░░░░ 80%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0556
  [Nodes] Active: 539 / 620
Gen  810/1000 | best=1572736.6113 | stale=11  | ████████████████░░░░ 81%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0556
  [Nodes] Active: 539 / 620
Gen  820/1000 | best=1572736.6111 | stale=0   | ████████████████░░░░ 82%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0556
  [Nodes] Active: 539 / 620
Gen  830/1000 | best=1572736.6109 | stale=1   | ████████████████░░░░ 83%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0554
  [Nodes] Active: 539 / 620
Gen  840/1000 | best=1572736.6107 | stale=2   | ████████████████░░░░ 84%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0553
  [Nodes] Active: 539 / 620
Gen  850/1000 | best=1572736.6105 | stale=5   | █████████████████░░░ 85%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0553
  [Nodes] Active: 539 / 620
Gen  860/1000 | best=1572736.6100 | stale=0   | █████████████████░░░ 86%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0550
  [Nodes] Active: 539 / 620
Gen  870/1000 | best=1572736.6100 | stale=7   | █████████████████░░░ 87%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0550
  [Nodes] Active: 539 / 620
Gen  880/1000 | best=1572736.6093 | stale=0   | █████████████████░░░ 88%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0546
  [Nodes] Active: 539 / 620
Gen  890/1000 | best=1572736.6093 | stale=10  | █████████████████░░░ 89%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0546
  [Nodes] Active: 539 / 620
Gen  900/1000 | best=1572736.6090 | stale=4   | ██████████████████░░ 90%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0545
  [Nodes] Active: 539 / 620
Gen  910/1000 | best=1572736.6090 | stale=1   | ██████████████████░░ 91%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0545
  [Nodes] Active: 539 / 620
Gen  920/1000 | best=1572736.6085 | stale=0   | ██████████████████░░ 92%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0543
  [Nodes] Active: 539 / 620
Gen  930/1000 | best=1572736.6085 | stale=3   | ██████████████████░░ 93%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0542
  [Nodes] Active: 539 / 620
Gen  940/1000 | best=1572736.6081 | stale=6   | ██████████████████░░ 94%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0541
  [Nodes] Active: 539 / 620
Gen  950/1000 | best=1572736.6080 | stale=0   | ███████████████████░ 95%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0540
  [Nodes] Active: 539 / 620
Gen  960/1000 | best=1572736.6077 | stale=0   | ███████████████████░ 96%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0539
  [Nodes] Active: 539 / 620
Gen  970/1000 | best=1572736.6077 | stale=4   | ███████████████████░ 97%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0538
  [Nodes] Active: 539 / 620
Gen  980/1000 | best=1572736.6075 | stale=5   | ███████████████████░ 98%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0537
  [Nodes] Active: 539 / 620
Gen  990/1000 | best=1572736.6073 | stale=0   | ███████████████████░ 99%
  [Scorecard] Cap: 0 | Sel: 0 | Gang: 0
              Frag: 1567313.50 | Aff: 0 | Var: 0.0537
  [Nodes] Active: 539 / 620
```

## stdout (Phase 0 pre-deduction, final blueprint)

```
═══ Phase 0: DaemonSet Pre-deduction (Ballast Water) ═══
  std-000: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-001: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-002: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-003: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-004: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-005: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-006: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-007: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-008: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-009: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-010: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-011: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-012: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-013: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-014: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-015: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-016: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-017: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-018: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-019: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-020: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-021: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-022: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-023: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-024: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-025: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-026: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-027: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-028: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-029: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-030: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-031: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-032: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-033: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-034: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-035: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-036: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-037: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-038: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-039: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-040: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-041: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-042: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-043: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-044: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-045: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-046: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-047: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-048: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-049: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-050: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-051: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-052: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-053: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-054: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-055: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-056: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-057: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-058: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-059: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-060: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-061: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-062: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-063: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-064: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-065: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-066: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-067: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-068: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-069: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-070: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-071: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-072: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-073: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-074: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-075: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-076: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-077: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-078: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-079: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-080: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-081: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-082: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-083: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-084: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-085: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-086: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-087: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-088: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-089: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-090: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-091: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-092: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-093: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-094: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-095: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-096: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-097: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-098: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-099: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-100: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-101: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-102: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-103: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-104: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-105: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-106: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-107: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-108: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-109: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-110: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-111: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-112: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-113: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-114: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-115: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-116: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-117: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-118: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-119: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-120: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-121: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-122: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-123: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-124: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-125: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-126: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-127: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-128: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-129: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-130: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-131: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-132: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-133: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-134: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-135: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-136: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-137: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-138: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-139: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-140: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-141: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-142: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-143: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-144: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-145: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-146: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-147: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-148: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-149: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-150: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-151: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-152: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-153: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-154: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-155: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-156: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-157: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-158: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-159: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-160: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-161: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-162: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-163: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-164: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-165: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-166: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-167: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-168: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-169: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-170: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-171: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-172: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-173: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-174: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-175: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-176: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-177: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-178: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-179: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-180: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-181: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-182: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-183: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-184: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-185: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-186: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-187: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-188: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-189: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-190: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-191: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-192: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-193: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-194: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-195: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-196: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-197: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-198: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-199: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-200: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-201: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-202: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-203: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-204: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-205: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-206: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-207: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-208: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-209: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-210: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-211: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-212: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-213: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-214: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-215: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-216: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-217: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-218: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-219: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-220: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-221: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-222: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-223: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-224: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-225: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-226: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-227: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-228: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-229: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-230: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-231: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-232: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-233: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-234: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-235: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-236: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-237: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-238: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-239: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-240: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-241: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-242: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-243: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-244: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-245: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-246: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-247: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-248: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-249: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-250: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-251: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-252: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-253: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-254: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-255: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-256: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-257: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-258: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-259: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-260: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-261: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-262: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-263: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-264: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-265: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-266: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-267: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-268: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-269: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-270: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-271: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-272: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-273: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-274: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-275: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-276: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-277: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-278: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-279: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-280: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-281: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-282: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-283: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-284: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-285: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-286: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-287: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-288: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-289: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-290: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-291: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-292: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-293: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-294: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-295: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-296: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-297: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-298: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-299: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-300: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-301: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-302: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-303: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-304: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-305: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-306: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-307: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-308: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-309: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-310: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-311: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-312: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-313: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-314: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-315: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-316: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-317: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-318: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-319: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-320: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-321: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-322: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-323: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-324: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-325: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-326: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-327: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-328: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-329: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-330: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-331: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-332: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-333: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-334: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-335: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-336: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-337: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-338: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-339: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-340: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-341: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-342: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-343: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-344: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-345: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-346: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-347: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-348: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-349: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-350: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-351: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-352: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-353: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-354: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-355: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-356: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-357: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-358: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-359: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-360: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-361: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-362: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-363: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-364: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-365: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-366: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-367: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-368: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-369: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-370: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-371: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-372: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-373: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-374: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-375: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-376: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-377: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-378: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-379: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-380: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-381: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-382: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-383: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-384: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-385: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-386: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-387: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-388: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-389: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-390: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-391: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-392: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-393: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-394: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-395: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-396: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-397: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-398: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  std-399: 64.0 → 61.75 CPU, 256.0 → 253.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-000: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-001: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-002: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-003: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-004: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-005: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-006: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-007: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-008: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-009: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-010: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-011: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-012: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-013: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-014: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-015: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-016: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-017: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-018: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-019: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-020: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-021: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-022: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-023: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-024: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-025: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-026: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-027: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-028: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-029: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-030: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-031: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-032: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-033: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-034: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-035: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-036: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-037: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-038: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-039: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-040: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-041: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-042: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-043: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-044: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-045: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-046: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-047: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-048: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-049: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-050: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-051: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-052: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-053: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-054: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-055: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-056: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-057: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-058: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-059: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-060: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-061: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-062: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-063: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-064: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-065: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-066: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-067: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-068: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-069: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-070: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-071: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-072: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-073: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-074: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-075: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-076: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-077: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-078: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-079: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-080: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-081: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-082: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-083: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-084: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-085: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-086: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-087: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-088: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-089: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-090: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-091: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-092: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-093: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-094: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-095: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-096: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-097: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-098: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-099: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-100: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-101: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-102: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-103: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-104: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-105: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-106: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-107: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-108: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-109: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-110: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-111: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-112: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-113: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-114: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-115: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-116: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-117: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-118: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  mem-119: 32.0 → 29.75 CPU, 512.0 → 509.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-000: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-001: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-002: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-003: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-004: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-005: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-006: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-007: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-008: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-009: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-010: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-011: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-012: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-013: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-014: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-015: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-016: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-017: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-018: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-019: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-020: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-021: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-022: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-023: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-024: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-025: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-026: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-027: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-028: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-029: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-030: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-031: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-032: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-033: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-034: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-035: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-036: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-037: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-038: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-039: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-040: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-041: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-042: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-043: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-044: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-045: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-046: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-047: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-048: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-049: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-050: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-051: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-052: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-053: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-054: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-055: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-056: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-057: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-058: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-059: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-060: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-061: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-062: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-063: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-064: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-065: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-066: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-067: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-068: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-069: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-070: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-071: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-072: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-073: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-074: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-075: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-076: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-077: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-078: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-079: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-080: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-081: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-082: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-083: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-084: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-085: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-086: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-087: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-088: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-089: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-090: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-091: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-092: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-093: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-094: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-095: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-096: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-097: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-098: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
  gpu-099: 48.0 → 45.75 CPU, 192.0 → 189.250 GiB RAM (-2.25 CPU, -2.750 GiB overhead)
═══ Final Blueprint (Stowage Plan) ═══
  Fitness: 1572736.6073
  Time: 912.49s
  Scorecard:
    Capacity Penalty: 0
    Selector Penalty: 0
    Gang Penalty: 0
    Active Nodes: 539
    Fragmentation: 1567313.50
    Affinity Violations: 0
    Utilization Variance: 0.0537
    Topology Spread Penalty: 11.00
  ─── Cluster Summary (620 nodes) ───
  Active nodes: 539 / 620
  Empty nodes:  81
  Pods placed:  2714
  Avg CPU util: 29.6%
  ─── Top 5 Busiest Nodes ───
  gpu-013: 44.0/45.8 CPU (96%), 9 pods
  mem-001: 28.0/29.8 CPU (94%), 9 pods
  gpu-009: 42.0/45.8 CPU (92%), 7 pods
  gpu-010: 42.0/45.8 CPU (92%), 7 pods
  gpu-011: 42.0/45.8 CPU (92%), 7 pods
  ─── Top 5 Lightest Active Nodes ───
  std-258: 6.0/61.8 CPU (10%), 2 pods
  std-160: 7.0/61.8 CPU (11%), 3 pods
  std-161: 7.0/61.8 CPU (11%), 3 pods
  std-162: 7.0/61.8 CPU (11%), 3 pods
  std-163: 7.0/61.8 CPU (11%), 3 pods
  ─── 81 nodes empty (available for shutdown) ───
  (Full stowage plan exported to kuberina_solution.yaml)
```

## `make inspector-run`

```
Loading infra: solver/testdata/irina_infra.yaml
Loading workloads: solver/testdata/irina_workloads.yaml
Loading solution: solver/kuberina_solution.yaml

--- Validation Results ---
Selector Violations: 0
Capacity Overflow (CPU): 0.00
Capacity Overflow (RAM): 0.00
Capacity Overflow (GPU): 0.00
Capacity Overflow (Storage): 0.00
Capacity Overflow (Disk Read): 0.00
Capacity Overflow (Disk Write): 0.00
Capacity Overflow (Net In): 0.00
Capacity Overflow (Net Out): 0.00
Unassigned Pods: 0
--------------------------

Heatmap dashboard generated: kuberina_dashboard.html
```

## `make bench-proof`

```
======================================================================
  KUBERINA MATHEMATICAL VERIFICATION
======================================================================
  Dataset: 620 nodes, 2714 pods
  Solution: 2714 assignments
  Headroom: none — the optimizer saw full capacity

──────────────────────────────────────────────────────────────────────
  PROOF 1: CONSTRAINT SATISFACTION (FEASIBILITY)
──────────────────────────────────────────────────────────────────────

  Predicate 1 — Capacity:
    ∀j ∈ N: Σᵢ xᵢⱼ · reqᵢʳ ≤ Cⱼʳ
    CPU overflow: 0.000000
    RAM overflow: 0.000000
    GPU overflow: 0.000000
    STORAGE overflow: 0.000000
    DISK_READ overflow: 0.000000
    DISK_WRITE overflow: 0.000000
    NET_IN overflow: 0.000000
    NET_OUT overflow: 0.000000
    Verdict: ✅ SATISFIED

  Predicate 2 — Assignment:
    ∀i ∈ P: Σⱼ xᵢⱼ = 1
    Missing pods: 0
    Verdict: ✅ SATISFIED

  Predicate 3 — Node Selector:
    ∀i: xᵢⱼ=1 ⟹ Selector(pᵢ) ⊆ Labels(nⱼ)
    Violations: 0
    Verdict: ✅ SATISFIED

  Objective — Topology Spread:
    Soft Penalty (Total Skew): 1100.00
    Verdict: ⚠️ SOFT PENALTY APPLIED

  ══ FEASIBILITY: ✅ PROVEN ══

──────────────────────────────────────────────────────────────────────
  PROOF 2: OPTIMALITY BOUND (LP RELAXATION)
──────────────────────────────────────────────────────────────────────

  ══ Bounds against real capacity ══

  ── Homogeneous LP Lower Bound (Coffman-Garey-Johnson 1978) ──
    L^cpu        = ⌈       7,198 /      61.75⌉ =  117
    L^ram        = ⌈      27,840 /     509.25⌉ =   55
    L^gpu        = ⌈         152 /       8.00⌉ =   19
    L^storage    = ⌈     109,410 /   1,991.50⌉ =   55
    L^disk_read  = ⌈     163,580 /   1,979.00⌉ =   83
    L^disk_write = ⌈     194,790 /   1,472.00⌉ =  133
    L^net_in     = ⌈     512,500 /   9,930.00⌉ =   52
    L^net_out    = ⌈     547,400 /   9,895.00⌉ =   56
    L = max(L^r) = 133

  ── Heterogeneous Utilization Bound ──
    ρ^cpu        = 0.2192   ⌈ρ·m⌉ =  136
    ρ^ram        = 0.1535   ⌈ρ·m⌉ =   96
    ρ^gpu        = 0.1900   ⌈ρ·m⌉ =  118
    ρ^storage    = 0.1598   ⌈ρ·m⌉ =  100
    ρ^disk_read  = 0.3104   ⌈ρ·m⌉ =  193
    ρ^disk_write = 0.5685   ⌈ρ·m⌉ =  353
    ρ^net_in     = 0.3210   ⌈ρ·m⌉ =  200
    ρ^net_out    = 0.3476   ⌈ρ·m⌉ =  216
    L_het = max(⌈ρʳ · m⌉) = 353

  Kuberina used: 539 active nodes
    α vs real-capacity bound     = 539/353 = 1.5269

  ══ α = 1.5269 — above FFD 1D guarantee, expected for multi-D ══

──────────────────────────────────────────────────────────────────────
  PROOF 3: STATISTICAL SIGNIFICANCE (MONTE CARLO)
──────────────────────────────────────────────────────────────────────

  Running 10000 random uniform trials...
    P(0 violations | random uniform) = 0.0
    Zero-violation trials: 0 / 10000
    Avg violations: 699.7 ± 18.6
    Range: [634, 770]

  Running 10000 selector-aware random trials...
    P(0 cap violations | selector-aware random) = 0.0
    Avg capacity violations: 439.4 ± 15.9
    Best random trial: 379 violations

  Kuberina: 0 violations
  Best random (selector-aware): 379 violations

  ══ P(random achieves Kuberina's result) < 1/10000 = 1.0e-04 ══
  ══ STATISTICALLY SIGNIFICANT: p < 1.0e-04 ══

======================================================================
  SUMMARY
======================================================================
  1. Feasibility:    PROVEN ✅
  2. Approx ratio:   α = 1.5269 (LB = 353 nodes, against real capacity)
  3. Significance:   p < 1.0e-04
======================================================================
```
