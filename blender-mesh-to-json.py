o = bpy.context.active_object

>>> list(o.data.vertices[62].groups)[1].weight
0.32011678814888

>>> list(o.data.vertices[62].groups)[1].group
6

>>> list(o.data.vertices[62].groups)[0].group
5

>>> o.vertex_groups
bpy.data.objects['temp-player-full-body'].vertex_groups

>>> o.vertex_groups[5]
bpy.data.objects['temp-player-full-body']...VertexGroup

>>> o.vertex_groups[5].name
'Lower.Arm.L'

>>> o.vertex_groups[6].name
'Hand.L'

