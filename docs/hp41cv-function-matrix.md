# HP-41CV ROM Function Matrix

> Generated from `docs/hp41cv-functions.json` via `just docs-matrix`.
> Edit the JSON, regenerate this file, commit both.

## Implemented (v2.x)

| Op | Display | Category | Status | Phase | Key Path | Description |
|----|---------|----------|--------|-------|----------|-------------|
| AlphaAppend | ALPHA char | Alpha | ✓ v2.x | 2 | — | Append character to ALPHA register (max 24 chars) |
| AlphaBackspace | ALPHA <- | Alpha | ✓ v2.x | 5 | — | Delete last character from ALPHA register |
| AlphaClear | CLRALPHA | Alpha | ✓ v2.x | 2 | — | Clear ALPHA register (legacy alias of CLA) |
| AlphaToggle | ALPHA | Alpha | ✓ v2.x | 2 | — | Toggle ALPHA mode (input/output text characters) |
| Arcl | ARCL | Alpha | ✓ v2.x | 23 | `f-a` | ARCL nn - append register's value to ALPHA |
| Arot | AROT | Alpha | ✓ v2.x | 23 | — | Rotate ALPHA by X chars (positive=left rotation) |
| Asto | ASTO | Alpha | ✓ v2.x | 23 | `f-A` | ASTO nn - pack first 6 ALPHA chars into reg nn |
| Atox | ATOX | Alpha | ✓ v2.x | 23 | — | Pop first ALPHA char, push its codepoint to X |
| Cla | CLA | Alpha | ✓ v2.x | 22 | — | Clear ALPHA register (hardware-faithful display name) |
| Posa | POSA | Alpha | ✓ v2.x | 23 | — | POSA - position of X (ASCII codepoint) in ALPHA |
| Xtoa | XTOA | Alpha | ✓ v2.x | 23 | — | Convert X mod 256 to char, append to ALPHA |
| Add | + | Arithmetic | ✓ v2.x | 1 | `+` | Add: X <- Y + X, drop stack |
| Div | / | Arithmetic | ✓ v2.x | 1 | `/` | Divide: X <- Y / X, drop stack |
| Mul | * | Arithmetic | ✓ v2.x | 1 | `*` | Multiply: X <- Y * X, drop stack |
| PctChange | %CH | Arithmetic | ✓ v2.x | 2 | `%` | Percent change from Y to X: ((X-Y)/Y)*100 |
| Sub | - | Arithmetic | ✓ v2.x | 1 | `-` | Subtract: X <- Y - X, drop stack |
| Rdprgm | RDPRGM | CardReader | ✓ v2.x | v2.1 | `XEQ "RDPRGM"` | Read program from card (name in ALPHA) |
| Rdta | RDTA | CardReader | ✓ v2.x | v2.1 | `XEQ "RDTA"` | Read data registers from card (name in ALPHA) |
| Wdta | WDTA | CardReader | ✓ v2.x | v2.1 | `XEQ "WDTA"` | Write data registers to card (name in ALPHA) |
| Wprgm | WPRGM | CardReader | ✓ v2.x | v2.1 | `XEQ "WPRGM"` | Write current program to card (name in ALPHA) |
| Catalog | CATALOG | Catalog | ✓ v2.x | 22 | — | CATALOG n - emit catalog n (1..4) to print buffer |
| HToHms | H->HMS | Conversion | ✓ v2.x | 6 | — | Convert decimal hours to H.MMSS |
| HmsAdd | HMS+ | Conversion | ✓ v2.x | 6 | — | Add two H.MMSS values (base-60 carry) |
| HmsSub | HMS- | Conversion | ✓ v2.x | 6 | — | Subtract H.MMSS values (base-60 borrow) |
| HmsToH | HMS->H | Conversion | ✓ v2.x | 6 | — | Convert H.MMSS to decimal hours |
| PolarToRect | P->R | Conversion | ✓ v2.x | 20 | — | Convert polar (Y=mag, X=angle) to rectangular |
| RectToPolar | R->P | Conversion | ✓ v2.x | 20 | — | Convert rectangular (Y=x, X=y) to polar |
| AView | AVIEW | Display | ✓ v2.x | 21 | — | Show ALPHA register on display (24-char truncate) |
| Aoff | AOFF | Display | ✓ v2.x | 21 | — | Disable ALPHA auto-display (clears flag 48) |
| Aon | AON | Display | ✓ v2.x | 21 | — | Enable ALPHA auto-display (sets system flag 48) |
| Cld | CLD | Display | ✓ v2.x | 21 | — | Explicit clear of display_override |
| FmtEng | ENG | Display | ✓ v2.x | 2 | `F` | ENG n - engineering notation display (n=0..9) |
| FmtFix | FIX | Display | ✓ v2.x | 2 | `F` | FIX n - fixed decimal display (n=0..9) |
| FmtSci | SCI | Display | ✓ v2.x | 2 | `F` | SCI n - scientific notation display (n=0..9) |
| Prompt | PROMPT | Display | ✓ v2.x | 21 | — | Show ALPHA on display; break execution in run_loop |
| SetDeg | DEG | Display | ✓ v2.x | 2 | — | Set angle mode to DEG |
| SetGrad | GRAD | Display | ✓ v2.x | 2 | — | Set angle mode to GRAD |
| SetRad | RAD | Display | ✓ v2.x | 2 | — | Set angle mode to RAD |
| View | VIEW | Display | ✓ v2.x | 21 | `f-v` | VIEW nn - show formatted value of register nn |
| CfFlag | CF | Flags | ✓ v2.x | 21 | `f-8` | CF n - clear flag n (0..55) |
| FlagTest | FS?/FC?/FS?C/FC?C | Flags | ✓ v2.x | 21 | `f-9` | Conditional flag test (skip next step on false) |
| SfFlag | SF | Flags | ✓ v2.x | 21 | `f-7` | SF n - set flag n (0..55) |
| ArclInd | ARCL IND | Indirect | ✓ v2.x | 24 | `f-a` | ARCL IND nn - append indirect register to ALPHA |
| AstoInd | ASTO IND | Indirect | ✓ v2.x | 24 | `f-A` | ASTO IND nn - pack ALPHA into indirect register |
| CfFlagInd | CF IND | Indirect | ✓ v2.x | 24 | `f-8` | CF IND nn - clear flag via indirect register addr |
| DseInd | DSE IND | Indirect | ✓ v2.x | 24 | `f-d` | DSE IND nn - decrement counter via indirect addr |
| FlagTestInd | FS?/FC?/FS?C/FC?C IND | Indirect | ✓ v2.x | 24 | `f-9` | Conditional flag test via indirect address |
| GtoInd | GTO IND | Indirect | ✓ v2.x | 22 | — | GTO IND nn - indirect branch through register nn |
| IsgInd | ISG IND | Indirect | ✓ v2.x | 24 | `f-i` | ISG IND nn - increment counter via indirect addr |
| RclInd | RCL IND | Indirect | ✓ v2.x | 24 | `R` | RCL IND nn - recall via indirect register address |
| SfFlagInd | SF IND | Indirect | ✓ v2.x | 24 | `f-7` | SF IND nn - set flag via indirect register address |
| StoArithInd | STO+/-/*// IND | Indirect | ✓ v2.x | 24 | `S` | STO arithmetic via indirect register address |
| StoInd | STO IND | Indirect | ✓ v2.x | 24 | `S` | STO IND nn - store via indirect register address |
| ViewInd | VIEW IND | Indirect | ✓ v2.x | 24 | `f-v` | VIEW IND nn - display value of indirect register |
| XeqInd | XEQ IND | Indirect | ✓ v2.x | 22 | — | XEQ IND nn - indirect subroutine call |
| Abs | ABS | Math | ✓ v2.x | 20 | — | Absolute value of X |
| ClSigmaStat | CL SIGMA | Math | ✓ v2.x | 6 | — | Clear stat registers R01..R06 to zero |
| Corr | CORR | Math | ✓ v2.x | 6 | — | Correlation coefficient r in X |
| Exp | E^X | Math | ✓ v2.x | 2 | — | Natural exponential of X |
| Fact | N! | Math | ✓ v2.x | 20 | — | Factorial of integer X; OutOfRange for X>69 |
| Frc | FRC | Math | ✓ v2.x | 20 | — | Fractional part of X (sign-preserving) |
| Int | INT | Math | ✓ v2.x | 2 | — | Truncate X toward zero (integer part) |
| LR | L.R. | Math | ✓ v2.x | 6 | — | Linear regression: slope m to Y, intercept b to X |
| Ln | LN | Math | ✓ v2.x | 2 | — | Natural logarithm of X |
| Log | LOG | Math | ✓ v2.x | 2 | — | Base-10 logarithm of X |
| Mean | MEAN | Math | ✓ v2.x | 6 | — | Push x-bar to X, y-bar to Y from stat regs |
| Mod | MOD | Math | ✓ v2.x | 20 | — | Y mod X with trunc-toward-zero convention |
| Pi | PI | Math | ✓ v2.x | 20 | — | Push pi onto X (3.141592654, 10-digit rounded) |
| Recip | 1/X | Math | ✓ v2.x | 2 | — | Reciprocal of X |
| Rnd | RND | Math | ✓ v2.x | 20 | — | Round X to precision of current display mode |
| Sdev | SDEV | Math | ✓ v2.x | 6 | — | Sample std dev (n-1): X<-sigma_x, Y<-sigma_y |
| SigmaMinus | SIGMA- | Math | ✓ v2.x | 6 | — | Remove X,Y from stat registers; push count |
| SigmaPlus | SIGMA+ | Math | ✓ v2.x | 6 | — | Accumulate X,Y into stat registers; push count |
| Sign | SIGN | Math | ✓ v2.x | 20 | — | Sign of X: -1, 0, or +1 |
| Sq | X^2 | Math | ✓ v2.x | 2 | — | Square of X |
| Sqrt | SQRT | Math | ✓ v2.x | 2 | — | Square root of X |
| TenPow | 10^X | Math | ✓ v2.x | 2 | — | Base-10 exponential of X |
| YPow | Y^X | Math | ✓ v2.x | 2 | — | Y raised to power X |
| Yhat | YHAT | Math | ✓ v2.x | 6 | — | y-hat prediction via linear regression |
| PRA | PRA | Print | ✓ v2.x | 11 | — | Print ALPHA register, left-aligned to 24 chars |
| PRSTK | PRSTK | Print | ✓ v2.x | 11 | — | Print full stack T/Z/Y/X/LASTX/ALPHA |
| PRX | PRX | Print | ✓ v2.x | 11 | — | Print X register, right-aligned to 24 chars |
| Asn | ASN | Programming | ✓ v2.x | 22 | — | ASN name key - assign name to key (USER mode) |
| Clp | CLP | Programming | ✓ v2.x | 22 | `f-C` | CLP name - clear program from LBL to next LBL |
| Del | DEL | Programming | ✓ v2.x | 22 | `f-D` | DEL nnn - delete nnn program steps starting at pc |
| Dse | DSE | Programming | ✓ v2.x | 3 | `f-d` | DSE nn - decrement counter, skip on underflow |
| Gto | GTO | Programming | ✓ v2.x | 3 | — | GTO name - unconditional branch to label |
| Ins | INS | Programming | ✓ v2.x | 22 | — | Insert Op::Null (no-op placeholder) at state.pc |
| Isg | ISG | Programming | ✓ v2.x | 3 | `f-i` | ISG nn - increment counter, skip on overflow |
| Lbl | LBL | Programming | ✓ v2.x | 3 | — | Program label marker (no-op at execution) |
| Pack | PACK | Programming | ✓ v2.x | 22 | — | Compact program memory (no-op in flat-Vec model) |
| PrgmMode | PRGM | Programming | ✓ v2.x | 3 | `p` | Toggle PRGM recording mode |
| Pse | PSE | Programming | ✓ v2.x | 22 | — | Pause: display X and emit ~1s delay marker |
| Rtn | RTN | Programming | ✓ v2.x | 3 | — | Return from subroutine |
| Size | SIZE | Programming | ✓ v2.x | 22 | — | SIZE nnn - resize state.regs to nnn (1..319) |
| Stop | STOP | Programming | ✓ v2.x | 22 | — | Halt program execution (breaks run_loop) |
| Test | X?Y / X?0 | Programming | ✓ v2.x | 3 | `f--` | Conditional test (skip next step on false) |
| UserMode | USER | Programming | ✓ v2.x | 5 | `u` | Toggle USER mode (key assignments active) |
| XGeY_XEQ | X>=Y? | Programming | ✓ v2.x | 25 | `XEQ "X>=Y?"` | X >= Y? conditional test (XEQ-by-Name only) |
| XGeZero_XEQ | X>=0? | Programming | ✓ v2.x | 25 | `XEQ "X>=0?"` | X >= 0? conditional test (XEQ-by-Name only) |
| XGtZero_XEQ | X>0? | Programming | ✓ v2.x | 25 | `XEQ "X>0?"` | X > 0? conditional test (XEQ-by-Name only) |
| XLeZero_XEQ | X<=0? | Programming | ✓ v2.x | 25 | `XEQ "X<=0?"` | X <= 0? conditional test (XEQ-by-Name only) |
| XLtY_XEQ | X<Y? | Programming | ✓ v2.x | 25 | `XEQ "X<Y?"` | X < Y? conditional test (XEQ-by-Name only) |
| XLtZero_XEQ | X<0? | Programming | ✓ v2.x | 25 | `XEQ "X<0?"` | X < 0? conditional test (XEQ-by-Name only) |
| XNeY_XEQ | X<>Y? | Programming | ✓ v2.x | 25 | `XEQ "X<>Y?"` | X != Y? conditional test (XEQ-by-Name only) |
| XNeZero_XEQ | X#0? | Programming | ✓ v2.x | 25 | `XEQ "X#0?"` | X != 0? conditional test (XEQ-by-Name only) |
| Xeq | XEQ | Programming | ✓ v2.x | 3 | `f-N` | XEQ name - subroutine call (max 4 deep) |
| Clreg | CLREG | Registers | ✓ v2.x | 2 | — | Clear all storage registers R00..R99 to zero |
| RclReg | RCL | Registers | ✓ v2.x | 2 | `R` | RCL nn - recall register nn into X (0..99) |
| StoArith | STO+/-/*// | Registers | ✓ v2.x | 9 | `S` | STO arithmetic on register nn using X |
| StoArithStack | STO+/-/*// Y/Z/T/L | Registers | ✓ v2.x | 9 | `S` | STO arithmetic on stack register using X |
| StoReg | STO | Registers | ✓ v2.x | 2 | `S` | STO nn - store X into register nn (0..99) |
| Beep | BEEP | Sound | ✓ v2.x | 21 | — | Push BEEP event to event_buffer |
| Tone | TONE | Sound | ✓ v2.x | 21 | `f-T` | TONE n - push TONE event (n=0..9) to buffer |
| Chs | CHS | Stack | ✓ v2.x | 1 | `n` | Change sign of X (negate) |
| Clst | CLST | Stack | ✓ v2.x | 22 | — | Clear stack X/Y/Z/T (preserves LASTX and lift) |
| Clx | CLX | Stack | ✓ v2.x | 1 | `BACKSPACE` | Clear X register (entry cancel) |
| Enter | ENTER | Stack | ✓ v2.x | 1 | `ENTER` | Lift stack and duplicate X into Y |
| Lastx | LASTX | Stack | ✓ v2.x | 1 | `l` | Recall last X (value before last operation) |
| Rdn | Rv | Stack | ✓ v2.x | 1 | `r` | Roll stack down: X<-Y, Y<-Z, Z<-T, T<-X |
| Rup | R^ | Stack | ✓ v2.x | 20 | — | Roll stack up: X<-T, T<-Z, Z<-Y, Y<-X |
| XySwap | X<>Y | Stack | ✓ v2.x | 1 | `x` | Swap X and Y registers |
| GetKey | GETKEY | Synthetic | ✓ v2.x | 12 | — | Push last key code (HP-41 row*10+col) to X |
| Null | NULL | Synthetic | ✓ v2.x | 12 | — | True no-op; does not modify any state |
| RclM | RCL M | Synthetic | ✓ v2.x | 12 | `R` | Recall hidden register M into X |
| RclN | RCL N | Synthetic | ✓ v2.x | 12 | `R` | Recall hidden register N into X |
| RclO | RCL O | Synthetic | ✓ v2.x | 12 | `R` | Recall hidden register O into X |
| StoM | STO M | Synthetic | ✓ v2.x | 12 | `S` | Store X to hidden register M |
| StoN | STO N | Synthetic | ✓ v2.x | 12 | `S` | Store X to hidden register N |
| StoO | STO O | Synthetic | ✓ v2.x | 12 | `S` | Store X to hidden register O |
| Acos | ACOS | Trig | ✓ v2.x | 2 | — | Arc cosine of X (result in current angle mode) |
| Asin | ASIN | Trig | ✓ v2.x | 2 | — | Arc sine of X (result in current angle mode) |
| Atan | ATAN | Trig | ✓ v2.x | 2 | — | Arc tangent of X (result in current angle mode) |
| Cos | COS | Trig | ✓ v2.x | 2 | — | Cosine of X (in current angle mode) |
| Sin | SIN | Trig | ✓ v2.x | 2 | — | Sine of X (in current angle mode) |
| Tan | TAN | Trig | ✓ v2.x | 2 | — | Tangent of X (in current angle mode) |

## v3.x Deferred (Module Pacs)

| Op | Display | Category | Status | Phase | Key Path | Description |
|----|---------|----------|--------|-------|----------|-------------|
| CMPLX_ADV | CMPLX | AdvantagePac | ⏳ v3.x module | — | — | Complex-number arithmetic (Advantage Pac, v3.x) |
| ITG_ADV | ITG | AdvantagePac | ⏳ v3.x module | — | — | Numerical integral (Advantage Pac, v3.x) |
| SOLVE_ADV | ADV SOLVE | AdvantagePac | ⏳ v3.x module | — | — | Equation solver (Advantage Pac, v3.x) |
| DET_MATHPAC | DET | MathPac | ⏳ v3.x module | — | — | Matrix determinant (Math Pac module, v3.x) |
| INTEG_MATHPAC | INTEG | MathPac | ⏳ v3.x module | — | — | Numerical integration (Math Pac, v3.x) |
| MAT_INV_MATHPAC | MAT INV | MathPac | ⏳ v3.x module | — | — | Matrix inversion (Math Pac module, v3.x) |
| MAT_MINUS_MATHPAC | MAT- | MathPac | ⏳ v3.x module | — | — | Matrix subtraction (Math Pac module, v3.x) |
| MAT_MUL_MATHPAC | MAT* | MathPac | ⏳ v3.x module | — | — | Matrix multiplication (Math Pac module, v3.x) |
| MAT_PLUS_MATHPAC | MAT+ | MathPac | ⏳ v3.x module | — | — | Matrix addition (Math Pac module, v3.x) |
| SOLVE_MATHPAC | SOLVE | MathPac | ⏳ v3.x module | — | — | Numerical root-finder (Math Pac, v3.x) |
| CHISQ_STATPAC | CHISQ | StatPac | ⏳ v3.x module | — | — | Chi-square distribution (Stat Pac, v3.x) |
| FCAT_STATPAC | FCAT | StatPac | ⏳ v3.x module | — | — | Frequency distribution catalog (Stat Pac, v3.x) |
| FCST_STATPAC | FCST | StatPac | ⏳ v3.x module | — | — | Statistical forecast (Stat Pac, v3.x) |
| ALARM_TIMEPAC | XYZALM | TimePac | ⏳ v3.x module | — | — | Schedule an alarm (Time module, v3.x) |
| DATE_PLUS_TIMEPAC | DATE+ | TimePac | ⏳ v3.x module | — | — | Add days to a date (Time module, v3.x) |
| DATE_TIMEPAC | DATE | TimePac | ⏳ v3.x module | — | — | Get current date (Time module, v3.x) |
| DDAYS_TIMEPAC | DDAYS | TimePac | ⏳ v3.x module | — | — | Days between two dates (Time module, v3.x) |
| TIME_TIMEPAC | TIME | TimePac | ⏳ v3.x module | — | — | Get current time (Time module, v3.x) |
