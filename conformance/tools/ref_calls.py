"""Full K24/K26 symbol interfaces; semantic callees retain ordinal spaces."""
def family(api):
    recipe=api['recipe'];out=[]
    for kind in ['lift.oracle_call','lift.oracle_uncall','lift.perm_call','oracle_call','action_call']:
        params=[{'name':'S','type':'symbol'},{'name':'A','type':'arguments','symbol':'S'}]
        body=[{'call':'S','args':'A'}]
        out.append(recipe(kind,params,body,'§4.2/§4.7 '+kind+': linked source symbol tier/signature and full positional interface are validated before expansion; preserve K24 values/tokens and K26 region shape/role/origin. Oracle uncall uses the identical involution; permutation/action inverse adjoints the callee.'))
    return out
