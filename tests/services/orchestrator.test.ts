import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const { invoke } = await import('@tauri-apps/api/core');
const { OrchestratorService, useConversation } = await import('../../src/lib/services/orchestrator');

describe('orchestrator service', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('getStats aggregates messages per slot', async () => {
    (invoke as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce(['a', 'b']) // listSlots
      .mockResolvedValueOnce([{ id: 1 }, { id: 2 }]) // getSlotMessages a
      .mockResolvedValueOnce([{ id: 3 }]); // getSlotMessages b

    const stats = await OrchestratorService.getStats();
    expect(stats.totalSlots).toBe(2);
    expect(stats.slots).toEqual([
      { id: 'a', messageCount: 2 },
      { id: 'b', messageCount: 1 },
    ]);
  });

  it('useConversation flow works', async () => {
    const convo = useConversation();
    (invoke as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce('slot-1') // create_slot
      .mockResolvedValueOnce('response') // send_message
      .mockResolvedValueOnce([{ id: 1 }]) // get_slot_messages
      .mockResolvedValueOnce(undefined) // clear_slot
      .mockResolvedValueOnce(true); // delete_slot

    const slotId = await convo.init(5);
    expect(slotId).toBe('slot-1');

    const response = await convo.send('hello');
    expect(response).toBe('response');

    const messages = await convo.getMessages();
    expect(messages).toEqual([{ id: 1 }]);

    await convo.clear();
    await convo.destroy();
    expect(convo.getSlotId()).toBeNull();
  });
});
