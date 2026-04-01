import { describe, it, expect, vi, beforeEach } from "vitest";
import { OrchestratorService, useConversation } from "$lib/services/orchestrator";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("OrchestratorService", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("createSlot calls invoke with correct params", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue("slot-123");

    const slotId = await OrchestratorService.createSlot(10);

    expect(invoke).toHaveBeenCalledWith("create_slot", { max_ctx: 10 });
    expect(slotId).toBe("slot-123");
  });

  it("createSlot uses default maxCtx", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue("slot-default");

    await OrchestratorService.createSlot();

    expect(invoke).toHaveBeenCalledWith("create_slot", { max_ctx: 10 });
  });

  it("deleteSlot calls invoke with slot_id", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue(true);

    const result = await OrchestratorService.deleteSlot("slot-123");

    expect(invoke).toHaveBeenCalledWith("delete_slot", { slot_id: "slot-123" });
    expect(result).toBe(true);
  });

  it("listSlots returns array of slot IDs", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue(["slot-1", "slot-2"]);

    const slots = await OrchestratorService.listSlots();

    expect(invoke).toHaveBeenCalledWith("list_slots");
    expect(slots).toEqual(["slot-1", "slot-2"]);
  });

  it("getSlotMessages returns messages array", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const messages = [
      { role: "user", content: "Hello" },
      { role: "assistant", content: "Hi" },
    ];
    vi.mocked(invoke).mockResolvedValue(messages);

    const result = await OrchestratorService.getSlotMessages("slot-123");

    expect(invoke).toHaveBeenCalledWith("get_slot_messages", {
      slot_id: "slot-123",
    });
    expect(result).toEqual(messages);
  });

  it("sendMessage calls invoke with all params", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue("AI response");

    const params = { temperature: 0.7, max_tokens: 256, top_p: 0.9, top_k: 40 };
    const result = await OrchestratorService.sendMessage("slot-1", "Hello", params);

    expect(invoke).toHaveBeenCalledWith("send_message", {
      slot_id: "slot-1",
      message: "Hello",
      params,
    });
    expect(result).toBe("AI response");
  });

  it("sendMessage works without params", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue("response");

    await OrchestratorService.sendMessage("slot-1", "Hi");

    expect(invoke).toHaveBeenCalledWith("send_message", {
      slot_id: "slot-1",
      message: "Hi",
      params: undefined,
    });
  });

  it("clearSlot calls invoke", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue(undefined);

    await OrchestratorService.clearSlot("slot-123");

    expect(invoke).toHaveBeenCalledWith("clear_slot", { slot_id: "slot-123" });
  });

  it("getStats aggregates slot information", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke)
      .mockResolvedValueOnce(["slot-1", "slot-2"])
      .mockResolvedValueOnce([{ role: "user", content: "msg1" }])
      .mockResolvedValueOnce([
        { role: "user", content: "msg2" },
        { role: "assistant", content: "msg3" },
      ]);

    const stats = await OrchestratorService.getStats();

    expect(stats.totalSlots).toBe(2);
    expect(stats.slots).toHaveLength(2);
    expect(stats.slots[0].messageCount).toBe(1);
    expect(stats.slots[1].messageCount).toBe(2);
  });

  it("getStats with no slots returns empty", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValueOnce([]);

    const stats = await OrchestratorService.getStats();

    expect(stats.totalSlots).toBe(0);
    expect(stats.slots).toEqual([]);
  });
});

describe("useConversation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("initializes a conversation slot", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue("slot-conv-1");

    const conv = useConversation();
    const slotId = await conv.init();

    expect(slotId).toBe("slot-conv-1");
    expect(conv.getSlotId()).toBe("slot-conv-1");
  });

  it("init accepts custom maxCtx", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue("slot-conv-2");

    const conv = useConversation();
    await conv.init(20);

    expect(invoke).toHaveBeenCalledWith("create_slot", { max_ctx: 20 });
  });

  it("send throws if not initialized", async () => {
    const conv = useConversation();

    await expect(conv.send("Hello")).rejects.toThrow(
      "Conversation not initialized"
    );
  });

  it("send works after init", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke)
      .mockResolvedValueOnce("slot-1")
      .mockResolvedValueOnce("AI reply");

    const conv = useConversation();
    await conv.init();
    const response = await conv.send("Hello");

    expect(response).toBe("AI reply");
  });

  it("send passes params", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke)
      .mockResolvedValueOnce("slot-1")
      .mockResolvedValueOnce("reply");

    const conv = useConversation();
    await conv.init();
    const params = { temperature: 0.5, max_tokens: 100, top_p: 0.8, top_k: 30 };
    await conv.send("test", params);

    expect(invoke).toHaveBeenCalledWith("send_message", {
      slot_id: "slot-1",
      message: "test",
      params,
    });
  });

  it("getMessages returns empty when not initialized", async () => {
    const conv = useConversation();
    const messages = await conv.getMessages();
    expect(messages).toEqual([]);
  });

  it("getMessages fetches from backend", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const mockMessages = [
      { role: "user", content: "Hi" },
      { role: "assistant", content: "Hello" },
    ];
    vi.mocked(invoke)
      .mockResolvedValueOnce("slot-1")
      .mockResolvedValueOnce(mockMessages);

    const conv = useConversation();
    await conv.init();
    const messages = await conv.getMessages();

    expect(messages).toEqual(mockMessages);
  });

  it("clear resets messages when initialized", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke)
      .mockResolvedValueOnce("slot-1")
      .mockResolvedValueOnce(undefined);

    const conv = useConversation();
    await conv.init();
    await conv.clear();

    expect(invoke).toHaveBeenCalledWith("clear_slot", { slot_id: "slot-1" });
  });

  it("clear is no-op when not initialized", async () => {
    const { invoke } = await import("@tauri-apps/api/core");

    const conv = useConversation();
    await conv.clear();

    expect(invoke).not.toHaveBeenCalled();
  });

  it("destroy deletes slot and resets state", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke)
      .mockResolvedValueOnce("slot-1")
      .mockResolvedValueOnce(true);

    const conv = useConversation();
    await conv.init();
    expect(conv.getSlotId()).toBe("slot-1");

    await conv.destroy();

    expect(invoke).toHaveBeenCalledWith("delete_slot", { slot_id: "slot-1" });
    expect(conv.getSlotId()).toBeNull();
  });

  it("destroy is no-op when not initialized", async () => {
    const { invoke } = await import("@tauri-apps/api/core");

    const conv = useConversation();
    await conv.destroy();

    expect(invoke).not.toHaveBeenCalled();
  });

  it("getSlotId returns null before init", () => {
    const conv = useConversation();
    expect(conv.getSlotId()).toBeNull();
  });
});
