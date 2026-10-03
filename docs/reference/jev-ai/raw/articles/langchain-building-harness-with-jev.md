---
source_url: https://www.langchain.com/blog/building-a-harness-with-jev
ingested: 2026-09-30
sha256: d3f4c3a7f16926b0fb456995bc1d9b06e74fc18478df7bd0a317aeb507332c16
---

Title: What Is Jev? A Guide to TypeSafe AI’s System One Model

URL Source: https://www.langchain.com/blog/building-a-harness-with-jev

Markdown Content:
## Building a Harness with Jev

Agents run in a loop: an LLM decides what to do, a tool executes, a model evaluates the results, and then continues in that loop until the task is complete.

Agents and LLMs were initially difficult to integrate into software applications, which depend on structured data and predictable interfaces. Two primitives emerged that made this much easier:

*   [**Tool calling**](https://openai.com/index/function-calling-and-other-api-updates/) let models make structured requests and receive structured results.
*   [**Structured outputs**](https://www.youtube.com/watch?v=yj-wSRJwrrc) let models return restructured results.

But even with those in place, the agent loop is still **slow** and **costly**: every decision requires another model call.

Enter, [Jev](https://x.com/CompleteSkeptic/status/2099925682726002904). Jev is a new model [released from TypeSafe AI](https://typesafe.ai/blog/introducing-system-one-models-and-jev). The company reports up to **200x faster inference and 400x lower cost** than comparable LLMs on classification tasks.

This post covers how Jev works, where it fits into the agent loop, and how to use it with LangChain.

### All about Jev

Jev is actually not a traditional LLM, it doesn’t generate text. It’s what the TypeSafe AI team calls a System One model:

> 📖 System One models are a class of AI models built to make fast, structured decisions that software can use directly. A System One model evaluates a [**state**](https://docs.typesafe.ai/concepts/state)and returns typed answers and probabilities.

It’s trained using [reinforcement learning for calibrated decisions (RLCD)](https://typesafe.ai/blog/introducing-system-one-models-and-jev). Your code uses those results to guide what an agent does next, without a full chat LLM call for each decision.

To invoke a Jev model, you send it a **state** (the context) and **questions** about that state. Here’s a single-question version of the support-ticket example in [their docs](https://docs.typesafe.ai/introduction/quickstart):`‍`

```
{
  "model": "jev-latest",
  "state": "Hi, I've been trying to connect my Stripe account for 3 days and it keeps failing. I'm losing sales. Please help ASAP.",
  "questions": {
    "is_urgent": {
      "type": "noul",
      "instructions": "The message conveys urgency or time-sensitivity"
    }
  }
}
```

The docs’ example gives this urgency answer, shown here without the rest of the response:`‍`

```
{
  "is_urgent": {
    "type": "noul",
    "noul": 0.999
  }
}
```

That’s a 99.9% probability that the message is urgent, which your application can use to prioritize the ticket.

There are three types of supported [questions](https://www.youtube.com/watch?si=L1qd4LT9W-W67mar&t=216&v=2Bs0Ink_-Uo&feature=youtu.be):

![Image 1](https://cdn.prod.website-files.com/65c81e88c254bb0f97633a71/6aac8b23a3980a735e102b7b_three_panels.png)

*   **Choice:** Pick from a set of options. Returns a probability for each option and an overall confidence score.
*   **Score:** Rate an input against ordered levels, such as low, medium, and high. Returns a continuous score, the underlying distribution, and a confidence value.
*   **Noul:** Answer a yes-or-no question. Returns the probability that a statement is true.

One key feature here is that you can ask multiple questions about the same state in one request.

> 💡 System One models evaluate every question in a request in parallel. Adding questions barely changes the response time and costs only the tokens for the extra questions, which are cheap.

For an example of asking multiple questions about a support ticket, see the [TypeSafe Quickstart](https://docs.typesafe.ai/introduction/quickstart#request-body).

In sum, unlike traditional LLMs, Jev is neither constrained by text generation or sequential decision making!

## How to Use Jev with LangChain

LangChain's provider agnostic model is well suited for supporting Jev alongside thousands of other integrations and model providers.

The [LangChain integration](https://docs.langchain.com/oss/python/integrations/providers/typesafe#quickstart) exposes Jev through `TypeSafeClassifier`. You pass your state and questions to `.invoke()`, and get classification results rather than a chat response.

Install `langchain-typesafe` and set your `TYPESAFE_API_KEY`, then make a call:`‍`

```
from langchain_typesafe import Noul, TypeSafeClassifier
 
classifier = TypeSafeClassifier()
 
response = classifier.invoke({
    "state": (
        "The deploy failed twice and customers are seeing 500s. "
        "Can someone look now?"
    ),
    "questions": {
        "urgent": Noul(
            instructions="Does this need attention right now?"
        ),
    },
})
 
urgency = response.nouls["urgent"].noul
```

The state can be text, structured data, or LangChain messages. That makes it straightforward to call Jev from a node or middleware hook using the context your agent already has.

You can build this into custom middleware or tools!

## Use Cases

Jev isn’t a drop-in replacement for an LLM. It doesn’t generate text, but it can handle classification tasks we often use LLMs for today, without the same latency and cost. That makes it a promising complement to the model driving your agent: use an LLM for open-ended reasoning and generation, and Jev for fast, structured decisions along the way.

### **Model routing**

A simple lookup doesn’t need the same model as a difficult debugging task. [Model-routing middleware](https://docs.langchain.com/oss/python/integrations/providers/typesafe#model-routing) lets Jev assess the request and choose a model based on criteria you define, so fast and inexpensive for straightforward tasks, more capable for complex ones.`‍`

```
from langchain.agents import create_agent
from langchain_typesafe.experimental.middleware import (
    ModelChoice,
    ModelRouterMiddleware,
)
 
router = ModelRouterMiddleware(
    choices={
        "fast": ModelChoice(
            model="openai:luna",
            criteria="Direct lookups, extraction, and localized changes.",
        ),
        "powerful": ModelChoice(
            model="openai:sol",
            criteria="Architecture and high-stakes decisions.",
        ),
    },
    instructions="Choose the least costly model that can complete the task.",
)
 
agent = create_agent("openai:gpt-5.6-luna", middleware=[router])
```

`‍`The router selects a model from the latest user message and uses it throughout the run. The probabilities and confidence remain available in agent state, too.

### **Auto Mode**

Agents are still inherently untrustworthy. An agent can receive bad instructions (either naturally or from a motivated enough attacker) which can persuade it into taking actions we didn’t want it to.

Coding harnesses like claude, codex, cursor have shipped some kind of way to classify dangerous actions _before_ they’re taken which has slowly helped to build trust in agents. Up until now, this classifier step has been locked away in the closed source parts of the harness.

Now that a cheap and performant classifier model exists, we can take the same pattern and adopt it to all agents!`‍`

```
from langchain.agents import create_agent
from langchain_typesafe.experimental.middleware import (
    AutoModeMiddleware,
)
 
guardrail = AutoModeMiddleware(tools=["bash"])
 
agent = create_agent("openai:gpt-5.6-luna", middleware=[guardrail])
```

`‍`[AutoModeMiddleware](https://docs.langchain.com/oss/python/integrations/providers/typesafe#tool-risk-gating) uses Jev to check tool calls for risky decisions it may take, and block calls before the tool executes.

## Get Started!

We're pretty thrilled about Jev and the possibilities that come with it. A few cool projects that we’ve seen already: [Kyle Jeong](https://x.com/kylejeong/status/2100622054945095934) from Browserbase is powering browser use agents for fractions of a cent, [Jarrod Watts](https://x.com/jarrodwatts/status/2100356151468585346) built a live trading agent, and [Ryan Vogel](https://x.com/ryanvogel/status/2100042788851101842) is doing email triage at scale.

New models drop every week at this point, but this one had a pretty outsized response. We’re excited to see what you build with LangChain and Jev.

Let us know what you think on the [forum](https://forum.langchain.com/), tag us on [X](https://x.com/LangChain?lang=en) and share what you’re building, or engage with [LangChain issues](https://github.com/langchain-ai/langchain)!

‍

### Related content

![Image 2](https://cdn.prod.website-files.com/65c81e88c254bb0f97633a71/6ab6ba616821ea2bd3edd56a_webflow-cover-light-building-prod-jev-langgraph-1600x900.png)

Open Source

Agent Architecture

LangGraph

#### Building Prod with Jev and LangGraph

![Image 3](https://cdn.prod.website-files.com/65c81e88c254bb0f97633a71/69dcee60745f0e15b18ad4d5_sydney-runkle.png)

![Image 4](https://cdn.prod.website-files.com/65c81e88c254bb0f97633a71/69e12735c02bb07c894a067a_hunter-lovell.png)

S. Runkle,

H. Lovell

September 25, 2026

![Image 5](https://cdn.prod.website-files.com/65b8cd72835ceeacd4449a53/69cd1fd0002272ce39bf1241_Icon-6.svg)

9

min

![Image 6](https://cdn.prod.website-files.com/65c81e88c254bb0f97633a71/6aa75f585bb405f1d1e0fc3b_arch-draft-dark-4880x2620.png)

Tutorials & How-Tos

Agent Architecture

#### How we built LangChain's Paid Media Agent

![Image 7](https://cdn.prod.website-files.com/65c81e88c254bb0f97633a71/6aa748ac8b001558f0c48c12_amal.jpeg)

![Image 8](https://cdn.prod.website-files.com/65c81e88c254bb0f97633a71/6aa7487b1796b28dfb325e62_danny.jpeg)

![Image 9](https://cdn.prod.website-files.com/65c81e88c254bb0f97633a71/6aa75ebed90e0d765a0890be_Screenshot%202026-09-13%20at%207.40.03%E2%80%AFPM.png)

A. Irgashev,

D. Lambert,

J. Gomez

September 13, 2026

![Image 10](https://cdn.prod.website-files.com/65b8cd72835ceeacd4449a53/69cd1fd0002272ce39bf1241_Icon-6.svg)

19

min

![Image 11](https://cdn.prod.website-files.com/65c81e88c254bb0f97633a71/6a9fa6750b484534373d853a_dark-57%20characters%20max%202.png)

Agent Architecture

Deep Agents

Open Source

#### Organizing Context in a Multi-Agent Harness

![Image 12](https://cdn.prod.website-files.com/65c81e88c254bb0f97633a71/6a9f9d4551b6d04251e14de9_T04F8K3FZB5-U0B7C5VFFL0-081c39c8d8b6-512.png)

![Image 13](https://cdn.prod.website-files.com/65c81e88c254bb0f97633a71/6a9fa3957a9e2fadccdce2d8_T04F8K3FZB5-U06PZQXSTQS-5c018c8b0d35-512.jpeg)

T. Bengre,

C. Curme

September 8, 2026

![Image 14](https://cdn.prod.website-files.com/65b8cd72835ceeacd4449a53/69cd1fd0002272ce39bf1241_Icon-6.svg)

6

min

![Image 15](https://cdn.prod.website-files.com/65b8cd72835ceeacd4449a53/69ce01ea562f8cc223cabf25_Frame%202147254328.svg)

### S

e

e

w

h

a

t

y

o

u

r

a

g

e

n

t

i

s

r

e

a

l

l

y

d

o

i

n

g

LangSmith, our agent engineering platform, helps developers debug every agent decision, eval changes, and deploy in one click.
