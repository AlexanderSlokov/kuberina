"""First rejected arrival index and the cluster GPU share arrived before it."""
import re, sys
workloads, blueprint, cluster_gpu = sys.argv[1], sys.argv[2], float(sys.argv[3])
decls, name = [], None
for line in open(workloads):
    if m := re.match(r"  - name: (\S+)", line): name, gpu = m.group(1), 0.0; decls.append([name, None, 0.0])
    elif m := re.match(r"    arrival: (\d+)", line): decls[-1][1] = int(m.group(1))
    elif m := re.search(r"gpu: ([\d.]+)", line): decls[-1][2] = float(m.group(1))
    elif m := re.match(r"    gpuMilli: (\d+)", line): decls[-1][2] *= int(m.group(1)) / 1000
unplaced = set(re.findall(r"^  - paib-gpu/(\S+)-0$", open(blueprint).read(), re.M))
arrived = 0.0
for name, arrival, gpu in sorted(decls, key=lambda d: d[1]):
    if name in unplaced:
        print(f"first rejection at arrival index {arrival}, {arrived / cluster_gpu * 100:.2f}% of cluster GPU arrived"); break
    arrived += gpu
