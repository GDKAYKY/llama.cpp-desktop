// ══════════════════════════════════════════════════════════════════
// COMPLETE EXAMPLE: Subagent Execution Trace
// ══════════════════════════════════════════════════════════════════

// USER QUERY:
// "Compare the weather in Tokyo and New York, and tell me which has better conditions for outdoor activities today"

// ══════════════════════════════════════════════════════════════════
// STEP 1: Intent Classification
// ══════════════════════════════════════════════════════════════════

// LLM Response:
{
  "needs_external": true,
  "query": "weather Tokyo New York comparison outdoor activities",
  "suggested_tool": "get_weather",
  "suggested_server": "weather_mcp",
  "arguments": null,  // Can't specify both locations in one call
  "needs_multi_step": true,
  "multi_step_reasoning": "Need to fetch weather for two different cities, then compare results"
}

// Orchestrator Decision: ACTIVATE SUBAGENT ✓

// ══════════════════════════════════════════════════════════════════
// STEP 2: Subagent Iteration 1
// ══════════════════════════════════════════════════════════════════

// Subagent Prompt:
{
  "role": "system",
  "content": "You are a data-gathering subagent. Your job is to collect all necessary information to answer the user's query by calling external tools.

Original query: weather Tokyo New York comparison outdoor activities

Available tools:
- weather_mcp.get_weather: Get current weather for a location
  Input: { location: string }
..."
}

// Subagent LLM Response (Iteration 1):
{
  "action_type": "call_tool",
  "reasoning": "First, get weather data for Tokyo",
  "tool_name": "get_weather",
  "server_id": "weather_mcp",
  "arguments": {
    "location": "Tokyo, Japan"
  }
}

// Tool Execution Result:
{
  "location": "Tokyo, Japan",
  "temperature": 18,
  "conditions": "Partly Cloudy",
  "humidity": 65,
  "wind_speed": 12,
  "precipitation": 0,
  "visibility": "Good"
}

// Added to Subagent Context:
{
  "role": "user",
  "content": "Tool called: get_weather from server weather_mcp
Arguments: { \"location\": \"Tokyo, Japan\" }
Result: { \"location\": \"Tokyo, Japan\", \"temperature\": 18, \"conditions\": \"Partly Cloudy\", ... }"
}

// ══════════════════════════════════════════════════════════════════
// STEP 3: Subagent Iteration 2
// ══════════════════════════════════════════════════════════════════

// Subagent LLM Response (Iteration 2):
{
  "action_type": "call_tool",
  "reasoning": "Now get weather data for New York to compare",
  "tool_name": "get_weather",
  "server_id": "weather_mcp",
  "arguments": {
    "location": "New York, USA"
  }
}

// Tool Execution Result:
{
  "location": "New York, USA",
  "temperature": 8,
  "conditions": "Light Rain",
  "humidity": 85,
  "wind_speed": 20,
  "precipitation": 5,
  "visibility": "Moderate"
}

// ══════════════════════════════════════════════════════════════════
// STEP 4: Subagent Iteration 3 - Finish
// ══════════════════════════════════════════════════════════════════

// Subagent LLM Response (Iteration 3):
{
  "action_type": "finish",
  "reasoning": "I have weather data for both cities, sufficient to compare outdoor conditions",
  "final_summary": "Weather comparison data collected:

Tokyo: 18°C, Partly Cloudy, 65% humidity, 12 km/h wind, no precipitation, good visibility
New York: 8°C, Light Rain, 85% humidity, 20 km/h wind, 5mm precipitation, moderate visibility

Tokyo has significantly better conditions for outdoor activities with warmer temperature, no rain, and better visibility."
}

// Subagent Result:
SubagentResult {
  data: [
    ToolCallResult {
      server_id: "weather_mcp",
      tool_name: "get_weather",
      arguments: { "location": "Tokyo, Japan" },
      result: { "temperature": 18, "conditions": "Partly Cloudy", ... }
    },
    ToolCallResult {
      server_id: "weather_mcp",
      tool_name: "get_weather",
      arguments: { "location": "New York, USA" },
      result: { "temperature": 8, "conditions": "Light Rain", ... }
    }
  ],
  summary: "Weather comparison data collected: Tokyo: 18°C, Partly Cloudy... New York: 8°C, Light Rain...",
  iterations_used: 3
}

// ══════════════════════════════════════════════════════════════════
// STEP 5: Inject Results into Main Orchestrator
// ══════════════════════════════════════════════════════════════════

// Formatted Data for Main LLM:
"=== EXTERNAL DATA GATHERED ===

Summary: Weather comparison data collected:
Tokyo: 18°C, Partly Cloudy, 65% humidity, 12 km/h wind, no precipitation, good visibility
New York: 8°C, Light Rain, 85% humidity, 20 km/h wind, 5mm precipitation, moderate visibility
Tokyo has significantly better conditions for outdoor activities.

Data collected from 2 tool call(s):

Source 1: get_weather from weather_mcp
Data: {
  \"location\": \"Tokyo, Japan\",
  \"temperature\": 18,
  \"conditions\": \"Partly Cloudy\",
  \"humidity\": 65,
  \"wind_speed\": 12,
  \"precipitation\": 0,
  \"visibility\": \"Good\"
}

Source 2: get_weather from weather_mcp
Data: {
  \"location\": \"New York, USA\",
  \"temperature\": 8,
  \"conditions\": \"Light Rain\",
  \"humidity\": 85,
  \"wind_speed\": 20,
  \"precipitation\": 5,
  \"visibility\": \"Moderate\"
}

=== END EXTERNAL DATA ==="

// ══════════════════════════════════════════════════════════════════
// STEP 6: Main LLM Generates Final Answer
// ══════════════════════════════════════════════════════════════════

// Main Orchestrator Prompt:
[
  {
    "role": "system",
    "content": "A specialized subagent has gathered the following external data for you.
Use this data to provide a comprehensive answer to the user's question.
Synthesize information from multiple sources if available.

[formatted data above]

Answer naturally in the user's language. Be thorough but concise."
  },
  {
    "role": "user",
    "content": "Compare the weather in Tokyo and New York, and tell me which has better conditions for outdoor activities today"
  }
]

// Main LLM Streaming Response:
"Based on current weather data, Tokyo has significantly better conditions for outdoor activities today:

**Tokyo:**
- Temperature: 18°C (comfortable)
- Conditions: Partly cloudy
- No precipitation
- Good visibility
- Moderate humidity (65%)
- Light winds (12 km/h)

**New York:**
- Temperature: 8°C (quite cold)
- Conditions: Light rain
- Active precipitation (5mm)
- Reduced visibility
- High humidity (85%)
- Strong winds (20 km/h)

**Recommendation:** Tokyo is the clear winner for outdoor activities. The pleasant temperature, dry conditions, and good visibility make it ideal for activities like walking, cycling, or outdoor sports. New York's rainy and cold conditions would make most outdoor activities uncomfortable."

// ══════════════════════════════════════════════════════════════════
// RESULT SENT TO USER ✓
// ══════════════════════════════════════════════════════════════════

// ══════════════════════════════════════════════════════════════════
// KEY OBSERVATIONS
// ══════════════════════════════════════════════════════════════════

// 1. Subagent made 2 tool calls sequentially
// 2. Each iteration was informed by previous results
// 3. Subagent's planning/reasoning never exposed to user
// 4. Main LLM received clean, structured data
// 5. Final answer synthesizes both data sources naturally
// 6. Total time: ~3-5 seconds (depending on LLM speed)

// ══════════════════════════════════════════════════════════════════
// CONTRAST WITH SINGLE-TOOL APPROACH
// ══════════════════════════════════════════════════════════════════

// Without subagent, you'd be forced to:
// - Pick one location only
// - OR require user to ask twice
// - OR use a hypothetical "batch weather" tool (if it exists)
// - Less flexible and more fragile

// With subagent:
// - Handles arbitrary complexity
// - Adapts to available tools dynamically
// - Works even if tools only accept one location at a time
// - Scales to 3, 4, 5+ locations without code changes
