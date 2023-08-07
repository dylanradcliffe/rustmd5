import re
import fileinput
# used to transform the ref impl of rounds from RFC1321 into the correct rust code
#FF (a, b, c, d, x[ 0], S11, 0xd76aa478); /* 1 */
#to
#Self::ff (&mut a, b, c, d, x[ 0], 7, 0xd76aa478); /* 1 */

#Self::op (&mut a, b, Self::aux_f(b, c, d), x[ 0], 7, 0xd76aa478); /* 1 */


S = [0] * 45

S[11]=7
S[12]=12
S[13]=17
S[14]=22
S[21]=5
S[22]=9
S[23]=14
S[24]=20
S[31]=4
S[32]=11
S[33]=16
S[34]=23
S[41]=6
S[42]=10
S[43]=15
S[44]=21

for line in fileinput.input():
    #FF (a, b, c, d, x[ 0], S11, 0xd76aa478); /* 1 */
    m = re.search(r'\s*(\w)\w \((.*)S(\d\d)(.*)', line)
    if m:
        fun = m.group(1).lower()
        va,vb,vc,vd,vx,_ = [s.strip() for s in m.group(2).split(",")]
        #Self::op (&mut a, b, Self::aux_f(b, c, d), x[ 0], 7, 0xd76aa478); /* 1 */
        print(f'Self::op (&mut {va}, {vb}, Self::aux_{fun}({vb}, {vc}, {vd}), {vx}, {str(S[int(m.group(3))]) + m.group(4)}' )
