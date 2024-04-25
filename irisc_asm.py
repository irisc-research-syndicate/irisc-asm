#!/usr/bin/env python3
import struct
import argparse
import jinja2
from pathlib import Path

def hexint(v): return int(v, 16)
def dec_or_hex(v):
    try:
        return int(v)
    except ValueError:
        return int(v, 16)

fields = {}

def field(func):
    fields[func.__name__] = func
    return func

def assert_sbits(v, n):
    assert -(2 ** (n-1)) <= v < (2 ** (n-1)), "Too many signed bits"
    return v & ((1 << n) - 1)

def assert_ubits(v, n):
    assert 0 <= v < 2 ** n, "Too many unsigned bits"
    return v

def reg(ctx, s):
    assert s[0] == 'r'
    return assert_ubits(int(s[1:]), 5)

@field
def rs(ctx, arg): return reg(ctx, arg) << 21

@field
def rd(ctx, arg): return reg(ctx, arg) << 16

@field
def rt(ctx, arg): return reg(ctx, arg) << 11

@field
def cmpop(ctx, arg): return assert_ubits(dec_or_hex(arg), 5) << 16

@field
def simm16(ctx, arg): return assert_sbits(dec_or_hex(arg), 16)

@field
def uimm16(ctx, arg): return assert_ubits(dec_or_hex(arg), 16)

@field
def opcode(ctx, arg): return assert_ubits(dec_or_hex(arg), 6) << 26

@field
def jmpop(ctx, arg): return assert_ubits(dec_or_hex(arg), 2)

@field
def rel24(ctx, arg):
    rel = ctx.labels.get(arg, 0) - ctx.address 
    assert rel & 3 == 0, "Unaligned rel24"
    return assert_sbits(rel >> 2, 24)

@field
def rel16(ctx, arg):
    rel = ctx.labels.get(arg, 0) - ctx.address 
    assert rel & 3 == 0, "Unaligned rel16"
    return assert_sbits(rel >> 2, 16)

@field
def off11(ctx, arg): return assert_ubits(dec_or_hex(arg), 11)

@field
def bitsel(ctx, arg): return assert_ubits(dec_or_hex(arg), 5) << 16

@field
def twobits(ctx, arg): return assert_ubits(dec_or_hex(arg), 2)

@field
def funct(ctx, arg): return assert_ubits(dec_or_hex(arg), 11)

def st(s):
    def func(ctx, args):
        inst = 0
        for part in s.split(' '):
            field, arg = part.split(":")
            if not arg: arg = args.pop(0)
            inst |= fields[field](ctx, arg)
        ctx.emit(inst)
    return func

def lbl_func(ctx, args):
    if args[0] in ctx.labels:
        assert ctx.labels[args[0]] == ctx.address
    else:
        ctx.labels[args[0]] = ctx.address

instructions = {
    "unk.r": st("opcode: rd: rs: rt: funct:"),
    "addi": st("opcode:0x00 rd: rs: simm16:"),
    "set0": st("opcode:0x06 rd: rs: uimm16:"),
    "set1": st("opcode:0x07 rd: rs: uimm16:"),
    "set3": st("opcode:0x08 rd: rs: uimm16:"),
    "set2": st("opcode:0x09 rd: rs: uimm16:"),
    "call": st("opcode:0x25 jmpop:0x0 rel24:"),
    "jump": st("opcode:0x25 jmpop:0x1 rel24:"),
    "alu.r": st("opcode:0x3f funct: rd: rs: rt:"),
    "add": st("opcode:0x3f rd: rs: rt: funct:0x000"),
    "sub": st("opcode:0x3f rd: rs: rt: funct:0x004"),
    "subs": st("opcode:0x3f rd: rs: rt: funct:0x005"),
    "alur.0xb": st("opcode:0x3f rd: rs: rt: funct:00b"),
    "b.t": st("opcode:0x28 cmpop: rs: rel16:"),
    "b.f": st("opcode:0x29 cmpop: rs: rel16:"),
    "b.set": st("opcode:0x2a rs: bitsel: rel16:"),
    "b.clr": st("opcode:0x2b rs: bitsel: rel16:"),
    "ld.d": st("opcode:0x19 rd: rs: rt: off11: twobits:0x2"),
    "st.d": st("opcode:0x1b rd: rs: rt: off11: twobits:0x2"),
    "st.q": st("opcode:0x1e rd: rs: rt: off11: twobits:"),
    "ret.d": st("opcode:0x3f rd: rs: rt: funct:0x02d"),
    "lbl": lbl_func
}

class Context:
    def __init__(self, base, labels={}):
        self.labels = dict(labels)
        self.base = base
        self.code = b''

    @property
    def address(self):
        return self.base + len(self.code)

    def emit(self, inst):
        self.code += struct.pack(">L", inst)

def assemble_pass(ctx, source):
    for line in source.split("\n"):
        line = line.strip()
        if (not line) or line.startswith("#"): continue
        op, args = line.split(" ", 1)
        args = [arg.strip() for arg  in args.strip().split(",")]
        instructions[op](ctx, args)

def assemble(source, base=0):
    ctx = Context(base)
    assemble_pass(ctx, source)
    ctx = Context(base, ctx.labels)
    assemble_pass(ctx, source)
    return ctx.code

def assemble_templated(path, args, base=0):
    env = jinja2.Environment(loader = jinja2.FileSystemLoader("."))
    tmpl = env.get_template(path)
    source = tmpl.render(args)
    return assemble(source, base=base)

def parse_args():
    p = argparse.ArgumentParser()
    #p.add_argument("--templated", default=False, action='store_true')
    p.add_argument("--int-arg", nargs=2, default=[], metavar=("key", "val"), action='append')
    p.add_argument("--str-arg", nargs=2, default=[], metavar=("key", "val"), action='append')
    p.add_argument("--base", type=hexint, default=0x00000000)
    p.add_argument("source", type=Path)
    p.add_argument("output", type=Path)
    return p.parse_args()

def main():
    args = parse_args()
    vars = dict(args.str_arg + [(k,int(v)) for k, v in args.int_arg])
    args.output.write_bytes(assemble_templated(str(args.source), vars, base=args.base))

if __name__ == '__main__': exit(main())

