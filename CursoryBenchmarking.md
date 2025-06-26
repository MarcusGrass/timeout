# Undetailed benchmarking

Syn parse + quote codegen => ~13micros  
manual parse + quote codegen => 3.5micros  
manual parse + mostly manual codegen => 2.5 micros  
manual parse + manual codegen => 2.35 micros  
Compeletely ditch syn-stack => ??? can no longer bench  
Reinstate proc_macro2 for tests => 1.69 micros  
