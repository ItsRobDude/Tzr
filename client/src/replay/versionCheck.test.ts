// client/src/replay/versionCheck.test.ts
import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { isReplayCompatible } from './versionCheck';

describe('isReplayCompatible', () => {
  it('accepts same major/minor with different patch', () => {
    assert.equal(isReplayCompatible('1.2.0', '1.2.3'), true);
  });

  it('accepts prerelease when core matches', () => {
    assert.equal(isReplayCompatible('1.2.0-alpha.1', '1.2.3'), true);
  });

  it('rejects different minor', () => {
    assert.equal(isReplayCompatible('1.2.0', '1.3.0'), false);
  });

  it('rejects different major', () => {
    assert.equal(isReplayCompatible('1.2.0', '2.2.0'), false);
  });

  it('treats malformed saved version as incompatible instead of throwing', () => {
    assert.equal(isReplayCompatible('not-a-version', '1.2.3'), false);
  });
});
