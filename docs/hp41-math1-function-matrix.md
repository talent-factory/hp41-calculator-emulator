# HP-41C Math Pac I Function Matrix

> Generated from `docs/hp41-math1-functions.json` via `just docs-matrix`.
> Edit the JSON, regenerate this file, commit both.

## Implemented (v2.x)

| Op | Display | XROM | Category | Status | Phase | Key Path | Description |
|----|---------|------|----------|--------|-------|----------|-------------|
| CDiv | C÷ | Math 1 / 7-10 | Math1 Complex Arithmetic | ✓ v2.x | 28 | `XEQ "C÷"` | Complex divide: (Yre+iYim) / (Xre+iXim) -> stack |
| CMinus | C- | Math 1 / 7-8 | Math1 Complex Arithmetic | ✓ v2.x | 28 | `XEQ "C-"` | Complex subtract: (Yre+iYim) - (Xre+iXim) -> stack |
| CPlus | C+ | Math 1 / 7-7 | Math1 Complex Arithmetic | ✓ v2.x | 28 | `XEQ "C+"` | Complex add: (Yre+iYim) + (Xre+iXim) -> stack (Y=re, X=im) |
| CTimes | C× | Math 1 / 7-9 | Math1 Complex Arithmetic | ✓ v2.x | 28 | `XEQ "C×"` | Complex multiply: (Yre+iYim) * (Xre+iXim) -> stack |
| Real | REAL | Math 1 / 7-11 | Math1 Complex Arithmetic | ✓ v2.x | 28 | `XEQ "REAL"` | Convert complex: real part in X, imaginary part in Y |
| ApowZ | A↑Z | Math 1 / 7-21 | Math1 Complex Functions | ✓ v2.x | 28 | `XEQ "A↑Z"` | Complex power A^Z: a^(Yre+iXre) with real base a in Z-register |
| Cinv | CINV | Math 1 / 7-13 | Math1 Complex Functions | ✓ v2.x | 28 | `XEQ "CINV"` | Complex reciprocal: 1/(Yre+iXre) -> stack (Y=re, X=im) |
| CosZ | COSZ | Math 1 / 7-19 | Math1 Complex Functions | ✓ v2.x | 28 | `XEQ "COSZ"` | Complex cosine: cos(Yre+iXre) -> stack (Y=re, X=im) |
| ExpZ | E↑Z | Math 1 / 7-16 | Math1 Complex Functions | ✓ v2.x | 28 | `XEQ "E↑Z"` | Complex exponential: e^(Yre+iXre) -> stack (Y=re, X=im) |
| LnZ | LNZ | Math 1 / 7-17 | Math1 Complex Functions | ✓ v2.x | 28 | `XEQ "LNZ"` | Complex natural log: ln(Yre+iXre) -> stack (Y=re, X=im) |
| LogZ | LOGZ | Math 1 / 7-22 | Math1 Complex Functions | ✓ v2.x | 28 | `XEQ "LOGZ"` | Complex base-10 log: log10(Yre+iXre) -> stack (Y=re, X=im) |
| Magz | MAGZ | Math 1 / 7-12 | Math1 Complex Functions | ✓ v2.x | 28 | `XEQ "MAGZ"` | Complex magnitude: \|z\| = sqrt(re^2+im^2), re in Y, im in X |
| SinZ | SINZ | Math 1 / 7-18 | Math1 Complex Functions | ✓ v2.x | 28 | `XEQ "SINZ"` | Complex sine: sin(Yre+iXre) -> stack (Y=re, X=im) |
| TanZ | TANZ | Math 1 / 7-20 | Math1 Complex Functions | ✓ v2.x | 28 | `XEQ "TANZ"` | Complex tangent: tan(Yre+iXre) -> stack (Y=re, X=im) |
| Zpow1N | Z↑1/N | Math 1 / 7-15 | Math1 Complex Functions | ✓ v2.x | 28 | `XEQ "Z↑1/N"` | Complex Nth root: (Yre+iZre)^(1/N) -> stack, N in X |
| ZpowN | Z↑N | Math 1 / 7-14 | Math1 Complex Functions | ✓ v2.x | 28 | `XEQ "Z↑N"` | Complex power: (Yre+iZre)^N -> stack, N in X (integer exponent) |
| ZpowW | Z↑W | Math 1 / 7-23 | Math1 Complex Functions | ✓ v2.x | 28 | `XEQ "Z↑W"` | Complex power Z^W: (Yre+iXre)^(Wre+iWim) -> stack |
| Trans2d | TRANS | Math 1 / 7-44 | Math1 Coordinate Transform | ✓ v2.x | 28 | `XEQ "TRANS"` | 2D coordinate transformation: rotate/translate point (x,y) by angle/offset |
| Trans3d | T3D | Math 1 / 7-45 | Math1 Coordinate Transform | ✓ v2.x | 28 | `XEQ "T3D"` | 3D coordinate transformation: rotate/translate point (x,y,z) in 3-space |
| Difeq | DIFEQ | Math 1 / 7-37 | Math1 Differential Eq | ✓ v2.x | 28 | `XEQ "DIFEQ"` | Differential equation solver: prompts for function name and step h |
| Four | FOUR | Math 1 / 7-38 | Math1 Fourier | ✓ v2.x | 28 | `XEQ "FOUR"` | Discrete Fourier transform (DFT) workflow on stored data vectors |
| Acosh | ACOSH | Math 1 / 7-5 | Math1 Hyperbolics | ✓ v2.x | 28 | `XEQ "ACOSH"` | Inverse hyperbolic cosine: X <- acosh(X) |
| Asinh | ASINH | Math 1 / 7-4 | Math1 Hyperbolics | ✓ v2.x | 28 | `XEQ "ASINH"` | Inverse hyperbolic sine: X <- asinh(X) |
| Atanh | ATANH | Math 1 / 7-6 | Math1 Hyperbolics | ✓ v2.x | 28 | `XEQ "ATANH"` | Inverse hyperbolic tangent: X <- atanh(X) |
| Cosh | COSH | Math 1 / 7-2 | Math1 Hyperbolics | ✓ v2.x | 28 | `XEQ "COSH"` | Hyperbolic cosine: X <- cosh(X) |
| Sinh | SINH | Math 1 / 7-1 | Math1 Hyperbolics | ✓ v2.x | 28 | `XEQ "SINH"` | Hyperbolic sine: X <- sinh(X) |
| Tanh | TANH | Math 1 / 7-3 | Math1 Hyperbolics | ✓ v2.x | 28 | `XEQ "TANH"` | Hyperbolic tangent: X <- tanh(X) |
| Integ | INTG | Math 1 / 7-34 | Math1 Integration | ✓ v2.x | 28 | `XEQ "INTG"` | Numerical integration: prompts for function name, lower/upper bounds |
| MatDet | DET | Math 1 / 7-30 | Math1 Matrix | ✓ v2.x | 28 | `XEQ "DET"` | Matrix determinant: det(A) -> X register |
| MatEdit | EDIT | Math 1 / 7-29 | Math1 Matrix | ✓ v2.x | 28 | `XEQ "EDIT"` | Edit matrix element A(i,j): prompts for row/col, enter new value |
| MatInv | INV | Math 1 / 7-31 | Math1 Matrix | ✓ v2.x | 28 | `XEQ "INV"` | Matrix inverse: A^-1 in place (non-singular square matrix required) |
| MatSimeq | SIMEQ | Math 1 / 7-32 | Math1 Matrix | ✓ v2.x | 28 | `XEQ "SIMEQ"` | Solve simultaneous linear equations: Ax=b, solution in x-vector |
| MatSize | SIZE | Math 1 / 7-27 | Math1 Matrix | ✓ v2.x | 28 | `XEQ "SIZE"` | Set matrix dimensions: SIZE rows x cols from X (packed: rows*100+cols) |
| MatVcol | VCOL | Math 1 / 7-33 | Math1 Matrix | ✓ v2.x | 28 | `XEQ "VCOL"` | View matrix column: display all elements in the selected column |
| MatVmat | VMAT | Math 1 / 7-28 | Math1 Matrix | ✓ v2.x | 28 | `XEQ "VMAT"` | View matrix: display all elements of the active matrix |
| MatrixWorkflow | MATRIX | Math 1 / 7-26 | Math1 Matrix | ✓ v2.x | 28 | `XEQ "MATRIX"` | Matrix workflow opener: prompts for ORDER=? (rows x cols) |
| PolyWorkflow | POLY | Math 1 / 7-24 | Math1 Polynomial | ✓ v2.x | 28 | `XEQ "POLY"` | Polynomial evaluation/root solver workflow (prompts for degree N) |
| Roots | ROOTS | Math 1 / 7-25 | Math1 Polynomial | ✓ v2.x | 28 | `XEQ "ROOTS"` | Find polynomial roots after POLY workflow completes coefficient entry |
| Sol | SOL | Math 1 / 7-36 | Math1 Root Solver | ✓ v2.x | 28 | `XEQ "SOL"` | Resume root-finding iteration (continue after SOLVE setup) |
| Solve | SOLVE | Math 1 / 7-35 | Math1 Root Solver | ✓ v2.x | 28 | `XEQ "SOLVE"` | Root-find workflow: prompts for function name and two initial guesses |
| TriAsa | ASA | Math 1 / 7-40 | Math1 Triangle Solvers | ✓ v2.x | 28 | `XEQ "ASA"` | Triangle solver: angle-side-angle — computes remaining sides and angle |
| TriSaa | SAA | Math 1 / 7-41 | Math1 Triangle Solvers | ✓ v2.x | 28 | `XEQ "SAA"` | Triangle solver: side-angle-angle — computes remaining sides and angle |
| TriSas | SAS | Math 1 / 7-42 | Math1 Triangle Solvers | ✓ v2.x | 28 | `XEQ "SAS"` | Triangle solver: side-angle-side — computes remaining sides and angles |
| TriSsa | SSA | Math 1 / 7-43 | Math1 Triangle Solvers | ✓ v2.x | 28 | `XEQ "SSA"` | Triangle solver: side-side-angle — ambiguous case handled (two solutions) |
| TriSss | SSS | Math 1 / 7-39 | Math1 Triangle Solvers | ✓ v2.x | 28 | `XEQ "SSS"` | Triangle solver: side-side-side — computes all angles from 3 sides |

## v3.x Deferred (Module Pacs)

_None._
