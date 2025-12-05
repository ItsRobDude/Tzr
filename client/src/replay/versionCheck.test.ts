import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { isReplayCompatible } from './versionCheck';

describe('isReplayCompatible', () => {
  it('accepts same major/minor, different patch', () => {
    assert.equal(isReplayCompatible('1.2.0', '1.2.3'), true);
  });

  it('rejects different minor', () => {
    assert.equal(isReplayCompatible('1.2.0', '1.3.0'), false);
  });

  it('rejects different major', () => {
    assert.equal(isReplayCompatible('1.2.0', '2.0.0'), false);
  });
});
