#!/bin/bash

# Check if a URL was provided
if [ -z "$1" ]; then
	echo "Usage: $0 <leetcode_url>"
	exit 1
fi

# Extract the problem slug from the end of the URL
# e.g., https://leetcode.com/problems/two-sum/ -> two-sum
SLUG=$(basename "$1")

# Use the robust GraphQL API method
curl -s 'https://leetcode.com/graphql' \
	-H 'Content-Type: application/json' \
	-d '{"query": "query questionTitle($titleSlug: String!) { question(titleSlug: $titleSlug) { questionId } }", "variables": {"titleSlug": "'$SLUG'"}}' | jq -r '.data.question.questionId'
