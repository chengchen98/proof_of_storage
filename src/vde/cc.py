import random
import time

NUM_BITS = 1024
NUM_ROUND = 100
time_total = 0

def fastModular(x): #快速幂的实现
	"""x[0] = base """
	"""x[1] = power"""
	"""x[2] = modulus"""
	result = 1
	while(x[1] > 0):
		if(x[1] & 1): # 位运算加快判断奇偶
			result = result * x[0] % x[2]
		x[1] = int(x[1]/2)
		x[0] = x[0] * x[0] % x[2]
	return result

for _ in range(NUM_ROUND):
    x = random.randrange(2 ** (NUM_BITS - 1), 2 ** NUM_BITS)
    p = 158297696608074679654124946564912202999139663277505984894261981349837992769596165683700437968679604111373729258655046764462137227577322861762501627230418997487671809885760928375348392323002752945263359796693275288611323927303851169352900910708127230034239565388759941444235878668699843286794016470366892082267
    tb = time.time()
    ans = fastModular([x,(p-1)//2,p])
    te = time.time()
    time_total += te-tb
    print(te-tb)

print("avg:",time_total/NUM_ROUND)