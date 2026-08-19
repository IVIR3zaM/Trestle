.PHONY: status graph graph-mermaid check

status:
	@python3 scripts/status.py

# The same graph drawn by layer: what can run in parallel, what each node needs
# and what it unlocks. Same script, same parser — a second reading of one source.
graph:
	@python3 scripts/status.py --graph

# A mermaid flowchart on stdout, for when the layer view isn't enough and you want
# boxes and arrows: `make graph-mermaid > /tmp/graph.md` and open it in any editor
# preview or paste it into a GitHub comment. No renderer here, and nothing leaves
# the machine unless you send it somewhere yourself.
graph-mermaid:
	@python3 scripts/status.py --mermaid

# The build graph's own consistency: node files, deps, tiers, gates and oracles
# all agreeing with graph.yaml. Cheap, and it catches the drift class that no
# other check sees — nothing reads a node file's frontmatter, so a stale `deps:`
# there misleads only the human or agent who opens it.
check:
	@bash scripts/check-plan-consistency.sh
