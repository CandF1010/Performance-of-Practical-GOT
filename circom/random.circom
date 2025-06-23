pragma circom 2.2.1;

template LargeCircuit(n){
	signal input x[n];
	signal output y[n];
	
	for (var i=0;i<n;i++){
		y[i]<==x[i]*x[i];
	}
}

component main = LargeCircuit(524288);
