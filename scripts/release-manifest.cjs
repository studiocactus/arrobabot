function publicManifest(manifest, release, version, repo) {
  if (manifest.version !== version) throw Error('Versão do manifesto diverge do pacote');
  if (!release.draft || release.tag_name !== 'v' + version) throw Error('É necessário um rascunho da versão correta');
  const prefix = `https://github.com/${repo}/releases/download/v${version}/`;
  const result = structuredClone(manifest);
  const platforms = Object.entries(result.platforms || {});
  if (!platforms.length) throw Error('Manifesto sem plataformas');
  for (const [key, platform] of platforms) {
    const asset = release.assets.find(a => a.url === platform.url || a.browser_download_url === platform.url || prefix + encodeURIComponent(a.name) === platform.url);
    if (!key.startsWith('windows-x86_64') || !platform.signature?.trim() || !asset ||
        !/\.(exe|msi)$/.test(asset.name) || !asset.browser_download_url.startsWith(`https://github.com/${repo}/releases/download/`) ||
        !release.assets.some(a => a.name === asset.name + '.sig')) {
      throw Error('Manifesto sem assinatura ou arquivo correspondente na release');
    }
    // Drafts may use an untagged-* URL until GitHub creates the release tag.
    platform.url = prefix + encodeURIComponent(asset.name);
  }
  return result;
}
module.exports = { publicManifest };
