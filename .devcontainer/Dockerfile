# Use the [Choice] comment to indicate option arguments that should appear in VS Code UX. Use a comma separated list.
FROM rust:1.68-bullseye

RUN export DEBIAN_FRONTEND=noninteractive 
#
# ****************************************************************************
# * TODO: Add any additional OS packages you want included in the definition *
# * here. We want to do this before cleanup to keep the "layer" small.       *
# ****************************************************************************
# && apt-get -y install --no-install-recommends <your-package-list-here> \
#

# [Optional] Uncomment this section to install additional OS packages.
# RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
#     && apt-get -y install --no-install-recommends <your-package-list-here>