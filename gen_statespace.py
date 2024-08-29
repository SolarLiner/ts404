import functools
import operator
from pathlib import Path

import sympy as sm
from lcapy import *
from valib.codegen import Visibility, SourceFile
from valib.statespace import StateSpace


def opamp_noninverting(zf, zg):
    return 1 + zf / zg


def create_discrete_statespace(hs: LaplaceDomainImpedance) -> DTStateSpace:
    hz = hs.bilinear_transform().simplify()
    return hz.ss


def statespace_input():
    pre = LSection(C(0.02e-6) + R(1e3), R(510e3) + V(4.5))
    post = Shunt(R(10e3)).chain(Series(C(1e-6)))
    h = (pre.Vtransfer.as_expr() * post.Vtransfer.as_expr()).simplify()
    return create_discrete_statespace(h)


def statespace_clipper():
    dist = symbol("pdist", real=True, positive=True)
    ff = LSection(C(1e-6), R(10e3)).Vtransfer.as_expr().simplify()
    fb = (R(500e3 * dist) + R(51e3)) | C(51e-9)
    fb = fb.Z.as_expr().simplify()
    fbg = C(0.047e-6) + R(4.7e3)
    fbg = fbg.Z.as_expr().simplify()

    # Explicitely applying gain (even through it should have been implicit in the complex impedances??)
    hs = (ff * opamp_noninverting(fb, fbg)).simplify() * lerp(12, 118, dist)
    return create_discrete_statespace(hs)


def tone_h_bass():
    bass_pre = LSection(R(1e3), C(0.22e-6)).chain(Shunt(C(0.22e-6) + R(220))).chain(Shunt(R(10e3) + V(4.5)))
    return bass_pre.Vtransfer.as_expr() * opamp_noninverting(1e3, sm.oo).as_expr()


def tone_h_treble():
    treble_pre = LSection(R(1e3), C(0.22e-6)).chain(Shunt(R(10e3) + V(4.5)))
    treble_gnd = C(0.22e-6) + R(220)
    return treble_pre.Vtransfer.as_expr() * opamp_noninverting(impedance(1e3), treble_gnd.Z.as_expr()).as_expr()


def lerp(a, b, t):
    return a + (b - a) * t


def g_taper(x):
    x = 2 * x - 1
    y = lerp(x, x ** 3, 0.75)
    return (y + 1) / 2


def statespace_tone():
    tone = symbol("ptone", real=True, positive=True)
    hs = lerp(tone_h_treble().as_expr(), tone_h_bass().as_expr(),g_taper(tone))
    return create_discrete_statespace(hs)


def statespace_output():
    pre = LSection(C(0.1e-6), R(510e3) + V(4.5))
    post = Shunt(R(10e3)).chain(Series(R(100) + C(1e-6))).chain(Shunt(R(10e3)))
    h = (pre.Vtransfer.as_expr() * post.Vtransfer.as_expr()).simplify()
    return create_discrete_statespace(h)


def codegen_full(*names: tuple[str, DTStateSpace]) -> SourceFile:
    def process(name: str, state_space: DTStateSpace):
        return StateSpace(state_space).as_source_file(name, Visibility.CRATE)

    return functools.reduce(operator.or_, (process(name, ss) for name, ss in names))


if __name__ == "__main__":
    source_file = codegen_full(("input", statespace_input()), ("clipper", statespace_clipper()),
                                               ("tone", statespace_tone()), ("output", statespace_output()))

    source_file.write_to(Path("src/gen.rs"))