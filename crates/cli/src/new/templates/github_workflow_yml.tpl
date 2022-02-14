name: CI

env:
  DEBUG: 'napi:*'
  APP_NAME: '{{ binary_name }}'
  MACOSX_DEPLOYMENT_TARGET: '10.13'

on:
  push:
    branches:
      - main
    tags-ignore:
      - '**'
    paths-ignore:
      - '**/*.md'
      - 'LICENSE'
      - '**/*.gitignore'
      - '.editorconfig'
      - 'docs/**'
  pull_request:

jobs:
  build:
    if: "!contains(github.event.head_commit.message, 'skip ci')"

    strategy:
      fail-fast: false
      matrix:
        settings: {% for target in targets %}
          - target: {{ target.triple }}
            host: {{ target.github_workflow_config.host }}
            architecture: {{ target.detail.arch }}
            {% if target.github_workflow_config.setup %}setup: {{ target.github_workflow_config.setup }}{% endif %}
            {% if target.github_workflow_config.docker %}docker: $DOCKER_REGISTRY_URL/{{ target.github_workflow_config.docker }}{% endif %}{% endfor %}

    name: stable - ${{ "{{" }} matrix.settings.target {{ "}}" }} - node@16
    runs-on: ${{ "{{" }} matrix.settings.host {{ "}}" }}
    steps:
      - uses: actions/checkout@v2
      - name: Setup node
        uses: actions/setup-node@v2
        with:
          node-version: 16
          check_latest: true
          cache: 'yarn'
          architecture: ${{ "{{" }} matrix.settings.architecture {{ "}}" }}

      - name: Install
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          override: true
          toolchain: stable
          target: ${{ "{{" }} matrix.settings.target {{ "}}" }}

      - name: Generate Cargo.lock
        uses: actions-rs/cargo@v1
        with:
          command: generate-lockfile

      - name: Cache cargo registry
        uses: actions/cache@v2
        with:
          path: ~/.cargo/registry
          key: ${{ "{{" }} matrix.settings.target {{ "}}" }}-node@16-cargo-registry-trimmed-{{ "{{" }} hashFiles('**/Cargo.lock') {{ "}}" }}

      - name: Cache cargo index
        uses: actions/cache@v2
        with:
          path: ~/.cargo/git
          key: ${{ "{{" }} matrix.settings.target {{ "}}" }}-node@16-cargo-index-trimmed-{{ "{{" }} hashFiles('**/Cargo.lock') {{ "}}" }}

      - name: Pull latest image
        run: |
          docker pull ${{ "{{" }} matrix.settings.docker {{ "}}" }}
          docker tag ${{ "{{" }} matrix.settings.docker {{ "}}" }} builder
        env:
          DOCKER_REGISTRY_URL: ghcr.io/napi-rs
        if: ${{ "{{" }} matrix.settings.docker {{ "}}" }}

      - name: Setup toolchain
        run: ${{ "{{" }} matrix.settings.setup {{ "}}" }}
        if: ${{ "{{" }} matrix.settings.setup {{ "}}" }}
        shell: bash

      - name: 'Install dependencies'
        run: yarn install

      - name: 'Build with custom image'
        run: docker run --rm -v ~/.cargo/git:/root/.cargo/git -v ~/.cargo/registry:/root/.cargo/registry -v $(pwd):/build -w /build builder yarn build --target ${{ "{{" }} matrix.settings.target {{ "}}" }} --strip
        shell: bash
        if: ${{ "{{" }} matrix.settings.docker {{ "}}" }} 

      - name: 'Build'
        run: yarn build --target ${{ "{{" }} matrix.settings.target {{ "}}" }} --strip
        shell: bash
        if: ${{ "{{" }} !matrix.settings.docker {{ "}}" }} 

      - name: Upload artifact
        uses: actions/upload-artifact@v2
        with:
          name: bindings-${{ "{{" }} matrix.settings.target {{ "}}" }}
          path: ${{ "{{" }} env.APP_NAME  {{ "}}" }}.*.node
          if-no-files-found: error
  
  {% for target in targets %}
  test-{{ target.triple }}:
    name: Test bindings on {{target.triple}} - node@-${{ "{{" }} matrix.node {{ "}}" }}
    needs:
      - build
    strategy:
      fail-fast: false
      matrix:
        node: ['12', '14', '16']
    runs-on: {{ target.github_workflow_config.host }}
    steps:
      - uses: actions/checkout@v2
      
      - name: Setup node
        uses: actions/setup-node@v2
        with:
          node-version: ${{ "{{" }} matrix.node {{ "}}" }}
          check-latest: true
          cache: 'yarn'
      
      - name: 'Install dependencies'
        run: yarn install

      - name: Download artifacts
        uses: actions/download-artifact@v2
        with:
          name: bindings-${{ "{{" }} matrix.settings.target {{ "}}" }}
          path: .

      - name: List packages
        run: ls -R .
        shell: bash

      - name: Test bindings
        run: docker run --rm -v $(pwd):/build -w /build node:${{ "{{" }} matrix.node {{ "}}" }}-slim yarn test
  {% endfor %}

  publish:
    name: Publish
    runs-on: ubuntu-latest
    needs:
      - build
      {% for target in targets %}- test-{{ target.triple }}
      {% endfor %}
    steps:
      - uses: actions/checkout@v2
      - name: Setup node
        uses: actions/setup-node@v2
        with:
          node-version: 16
          check-latest: true
          cache: 'yarn'
      - name: 'Install dependencies'
        run: yarn install

      - name: Download all artifacts
        uses: actions/download-artifact@v2
        with:
          path: artifacts

      - name: Move artifacts
        run: yarn artifacts

      - name: List packages
        run: ls -R ./npm
        shell: bash

      - name: Publish
        run: |
          if git log -1 --pretty=%B | grep "^[0-9]\\+\\.[0-9]\\+\\.[0-9]\\+$";
          then
            echo "//registry.npmjs.org/:_authToken=$NPM_TOKEN" >> ~/.npmrc
            npm publish --access public
          elif git log -1 --pretty=%B | grep "^[0-9]\\+\\.[0-9]\\+\\.[0-9]\\+";
          then
            echo "//registry.npmjs.org/:_authToken=$NPM_TOKEN" >> ~/.npmrc
            npm publish --tag next --access public
          else
            echo "Not a release, skipping publish"
          fi
        env:
          GITHUB_TOKEN: ${{ "{{" }} secrets.GITHUB_TOKEN {{ "}}" }}
          NPM_TOKEN: ${{ "{{" }} secrets.NPM_TOKEN {{ "}}" }}
