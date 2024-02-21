#!/bin/bash

# Default GitHub username
DEFAULT_USERNAME="ManReadApp"

# Array of repository names
REPOS=("api" "app" "api_structure" "ethread" "scraper")

# Function to clone repositories
clone_repos() {
    local username="${1:-$DEFAULT_USERNAME}"

    # Iterate over each repository in the array
    for repo in "${REPOS[@]}"; do
        # Construct the GitHub URL with the username and repository name
        url="https://github.com/$username/$repo.git"

        # Clone the repository
        git clone "$url"

        # Check if clone was successful
        if [ $? -eq 0 ]; then
            echo "Repository '$repo' cloned successfully."
        else
            echo "Failed to clone repository '$repo'."
        fi
    done
}

# Main execution
main() {
    # Check if any arguments are provided, assuming the first argument is the username
    if [ $# -gt 0 ]; then
        clone_repos "$1"
    else
        clone_repos "$DEFAULT_USERNAME"
    fi
}

# Execute the main function
main "$@"
