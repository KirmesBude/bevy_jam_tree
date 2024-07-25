- Implement per season start and end logic (e.g. growing/dieing)
- Implement growing and dying (among other things)


Current game plan:
Gain points for each felled tree (mature and overmature)
Lose points for each seedling and overmature tree that dies (not burned)
Lose points for each felled tree (immature?)
Burning gives not point but turns >=mature trees to good soil -> felled tree on good soil gives 3x points

Spring: Can place x amount of seedlings any free tile; normal growth
Summer: Can place x amount of fire tiles (Fire spreads to adjacent tree tiles, can jump once?); Always growing/no dying?
Autumn: Can place x amount of gust tiles that will create seedlings in that direction; normal growth
Winter: Can place x amount of storm tiles so they are not cut yet; no growth, seedlings die; All overmature/mature trees are felled, but minimum of 4 of any kind? (good idea?)

Tree logic:
0-2   neighbour level -> normal growth
3-4   neighbour level -> stop growing
5-8   neighbour level -> die

Tree Level: (1, 1, 1, 2)