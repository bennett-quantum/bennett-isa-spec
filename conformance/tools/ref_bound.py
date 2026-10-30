"""Typed row and structured constructor catalog. No private runtime leaves."""
def formal(name,ty='integer',width=None):
    f={'name':name,'type':ty}
    if width is not None:f['width']=width
    return f
CLASSICAL=['const','add','sub','mul','and','or','xor','shl','lshr','ashr','udiv','urem','sdiv','srem','icmp.eq','icmp.ne','icmp.ult','icmp.ule','icmp.slt','icmp.sle','select','zext','sext','trunc','ptrtoint','inttoptr','load','store','alloc','free','br','call','ret','uncall','trap','fault','qreg.new','qreg.free','qreg.slice','qreg.concat','qreg.width','qreg.id','qreg.from_id','measure','measure.iN','budget.mint','effect','hostcall']
def classical(api):
    recipe=api['recipe']
    return [recipe('cc.'+op,[formal('E','event')],[{'classical':op,'point':'E'}], '0.9 §4.1: '+op+'; classical event at the bound §8.2 point, retaining §7.4 history.') for op in CLASSICAL]
