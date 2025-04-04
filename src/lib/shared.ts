import type { KeyInfo } from './types.ts';

type View = 'list' | 'generate' | 'details' | 'sign' | 'verify';
export const shared: { currentView: View; title: string; keys: KeyInfo[] } = $state({
  currentView: 'list',
  title: 'Key Management',
  keys: []
});

export const lastSelectedKey: { index: number; info: KeyInfo } | null = $derived.by(() => {
  const keys = shared.keys;
  if (!lastSelectedKey || keys.length == 0 || keys.length <= lastSelectedKey.index) {
    return null;
  }

  const newLastSelectedKey = keys[lastSelectedKey.index];
  if (newLastSelectedKey.keyId != lastSelectedKey.info.keyId) {
    return null;
  }

  return { index: lastSelectedKey.index, info: newLastSelectedKey };
});
