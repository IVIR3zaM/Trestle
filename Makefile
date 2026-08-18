.PHONY: status check

status:
	@python3 scripts/status.py

# The build graph's own consistency: node files, deps, tiers, gates and oracles
# all agreeing with graph.yaml. Cheap, and it catches the drift class that no
# other check sees — nothing reads a node file's frontmatter, so a stale `deps:`
# there misleads only the human or agent who opens it.
check:
	@bash scripts/check-plan-consistency.sh
