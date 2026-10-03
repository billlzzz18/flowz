---
source_url: https://www.browserbase.com/blog/what-is-jev
ingested: 2026-09-30
sha256: 2ff7ea7690ebe37fd45eaac8d74fc4aa2ebcdc41714c3215582f6f8d1fe60c30
---

---
title: "What is Jev? | Browserbase"
canonical_url: "https://browserbase.com/blog/what-is-jev/"
---

September 21, 2026

# What is Jev?

![](https://cdn.sanity.io/images/yd6zslid/production/95c585473bfcf3c8eb260a3b9418033dbcce9ee9-552x556.png?fm=webp&h=1612&w=1600&auto=format&fit=min&q=75)

Kyle Jeong

[Engineering](/blog/engineering/)

7 min read

![Jev article cover](https://cdn.sanity.io/images/yd6zslid/production/cd3a1c87bc3f777a2a15455ef582259e571d2432-856x579.png?fm=webp&h=1082&w=1600&auto=format&fit=min&q=75)

Unless you’ve been living under a rock for the past week, you’ve seen Jev from [@typesafeai](https://x.com/typesafeai).

> [View post on X](https://twitter.com/CompleteSkeptic/status/2099925682726002904)

They describe their models as:

a class of AI models built to make fast, structured decisions that software can use directly. A System One model evaluates a [state](https://docs.typesafe.ai/concepts/state) and returns typed answers and probabilities.

According to Typesafe's benchmarks, Jev is 20-200x faster and 40-400x cheaper than LLMs.

But why does this even matter? We’ve trained classifiers before (autocorrect in your phone, gmail filtering, etc), but according to twitter Jev is something special.

In this article, I aim to teach you what Jev is, why it exists, and how you can introduce it into your production systems.

## So what is Jev exactly?

[“Think of Jev as a frontier-intelligence function call: unstructured state in, typed probabilistic decisions out.”](https://typesafe.ai/blog/introducing-system-one-models-and-jev) - Typesafe

Jev exposes 3 primitives: [Choice](https://docs.typesafe.ai/primitives/choice), [Score](https://docs.typesafe.ai/primitives/score), and [Noul](https://docs.typesafe.ai/primitives/noul).

-   Choice is a question type that selects one option from a defined set (of max 255) whose answer includes the selected option, a probability for each option, and confidence.
-   Score rates content against ordered, descriptive levels whose answer includes a score, a probability for each level, and confidence.
-   Noul asks the model to evaluate a yes/no question and return the probability that the answer is yes.

Here’s a sample input and output for a customer-support use case:

```text
// Input
{
  "model": "jev-latest",
  "state": "Hi, I was charged twice for my monthly subscription. Could you refund the extra charge? My account is working fine.",
  "questions": {
    "department": {
      "type": "choice",
      "instructions": "Which team should handle this message?",
      "criteria": {
        "billing": "Charges, payments, subscriptions, and refunds",
        "technical": "Bugs, errors, and broken features",
        "account": "Login, passwords, and account access"
      }
    },
    "requests_refund": {
      "type": "noul",
      "instructions": "Is the customer explicitly requesting a refund?"
    },
    "frustration": {
      "type": "score",
      "instructions": "How frustrated does the customer sound?",
      "criteria": [
        "Calm: politely describes the issue without expressing frustration",
        "Frustrated: expresses annoyance or dissatisfaction",
        "Very frustrated: expresses strong anger or threatens to leave"
      ]
    }
  }
}
// Output
{
  "model": "jev-1.13.0",
  "answers": {
    "department": {
      "type": "choice",
      "choice": "billing",
      "confidence": 1,
      "probabilities": {
        "technical": 0,
        "account": 0,
        "billing": 1
      }
    },
    "requests_refund": {
      "type": "noul",
      "noul": 0.99
    },
    "frustration": {
      "type": "score",
      "score": 0,
      "legend": {
        "0": "Calm: politely describes the issue without expressing frustration",
        "1": "Frustrated: expresses annoyance or dissatisfaction",
        "2": "Very frustrated: expresses strong anger or threatens to leave"
      },
      "confidence": 1,
      "probabilities": {
        "0": 1,
        "1": 0,
        "2": 0
      }
    }
  },
  "usage": {
    "input_tokens": 442,
    "output_tokens": 72
  },
  "request_id": "playground_12bbfa4198be5ca4de9818a45c0906a2055",
  "evaluation_time_ms": 163.01120699790772
}
```

I’d recommend going through their [dashboard onboarding](https://console.typesafe.ai/playground) for a better understanding of how state and questions work together for outputs.

## Isn’t this just a classifier?

Well yes and no. It’s more like if an LLM and a classifier had a baby.

Traditional classifiers are good for high-volume, fixed taxonomy tasks. Think [LeNet-5](https://en.wikipedia.org/wiki/LeNet) being able to identify what digit an image represented. However classifiers are usually ultra-specialized and domain specific. LLMs are good generating sequences. Their flexible and great for open-ended tasks defined at runtime, but they’re also slower (than a classifier), more expensive, and less predictable.

Jev is a “foundation model for classification”, combining the natural-language flexibility of an LLM with the constrained, probabilistic output of a classifier. You can complete a wide variety of tasks without having to train a new model, while also maintaining expertise across variety of different domains (code, text, logs, UI states, events, etc.). Jev can also generate output in parallel, which makes it much faster than an LLM which is limited to sequential generation.

TL;DR it’s a really smart generalizable classifier.

## Why does Jev exist?

Typesafe’s CEO and co-founder [Diogo Almeida](https://www.linkedin.com/in/diogomda/) spent time at OpenAI helping create RLHF (reinforcement learning from human feedback) and the ChatGPT product. RLHF let us train LLMs to be really good at following instructions and prompts, which matches their autoregressive nature.

He then left OpenAI to start TypeSafe and train a different class of models to enable AI-powered software, rather than agents. Jev is trained with RLCD (reinforcement learning from calibrated decisions) which is research speak for they train the model to be really good at outputting confidence and probabilities rather than answers.

Typesafe believes is that [software should be intelligent](https://typesafe.ai/manifesto). Agents don’t naturally diffuse into how software historically worked, and human-in-the-loop makes it hard for intelligent AND autonomous software. Jev is a step towards intelligence as a composable, dependable primitive inside software systems.

This isn’t a completely new idea, researchers in 2017 found that [strong predictive accuracy doesn’t mean reliable confidence estimates](https://proceedings.mlr.press/v70/guo17a.html) (so LLMs aren’t a perfect solution here).

## Why RLCD over RLHF?

The problem with RLHF is that what humans want isn’t always objectively correct. Just because we prefer a certain answer in a certain format doesn’t make the models more intelligent, but rather nicer to work with.

It also introduces mode collapse, RLHF makes LLMs converge to a single answer where sometimes multiple trajectories could be “correct”.

[“An output can be compelling to a person without being reliable enough for unattended automation. Human preference and machine trustworthiness are](https://docs.typesafe.ai/introduction/machine-learning-primer) [different optimization targets.”](https://docs.typesafe.ai/introduction/machine-learning-primer)

![RLHF mode collapse: probability mass concentrates around one reward-favored output while other modes disappear.](https://cdn.sanity.io/images/yd6zslid/production/0bccd003d1698d0883c7b0b44234224b6e882b1b-789x449.png?fm=webp&h=911&w=1600&auto=format&fit=min&q=75)

_Mode collapse via Jev's Docs_

## Jev was NOT meant to build agents

Contrary to what you’re seeing all over your timeline, Jev is not very good as a standalone agent. We’ve tried to build versions of it, both Jev only and LLM + Jev.

> [View post on X](https://twitter.com/kylejeong/status/2100622054945095934)

Honestly, Jev agents do make for cool demos. There’s been a ton of them using Jev to do agent tasks at lightning speed. But even the best demos aren’t ready to be deployed to production.

A model like Jev is meant for AI-powered software, it can help you make decisions composed in deterministic code. Without reasoning or generative capabilities, using it as a standalone agent is just pure ignorance.

![Traditional software, agents, and AI-powered software compared as decision and execution flows.](https://cdn.sanity.io/images/yd6zslid/production/da4175b03b2f518445bdfa0ceb39196a8d56bc6c-778x442.png?fm=webp&h=909&w=1600&auto=format&fit=min&q=75)

_AI-powered Software_

Rather than let Jev be a standalone computer use agent it should be used in customer support routing, invoice processing, security alerts and triaging, or as an agent monitor.

## The numbers

Their first model Jev 1.13.0 costs $0.042/mtok input and $0 for output tokens. The context window is 64k tokens per request, where state + the longest question must fit in 32k tokens.

In their internal benchmarks however they outperform all models in accuracy/cost & accuracy/speed from OpenAI, Anthropic, and Deepseek (via Fireworks for inference).

![Mean accuracy versus cost per case comparing Jev and LLM prompt and workflow benchmarks.](https://cdn.sanity.io/images/yd6zslid/production/c2c5561606dbca628a4f7ac83967f6d816e56e25-811x517.png?fm=webp&h=1020&w=1600&auto=format&fit=min&q=75)

_accuracy/cost_

## Enough talk, how do I use it?

You should now have enough of an understanding of Jev to have thought of a few use cases whatever you’re currently working on. [(If not here’s a list of use cases recommended by Typesafe)](https://docs.typesafe.ai/concepts/use-case-map).

Rather than limit your creativity on how you should use it, I’ll show you how we’ve retrofitted Jev into our framework Stagehand.

For the last 2 years [Stagehand](https://github.com/browserbase/stagehand) has evolved as a framework for AI and Agents to control a remote browser. Before agents were good enough, we create AI-primitives Act (complete an action), Extract (pull structured data), and Observe (discover potential actions on a page) to help developers write self-healing scripts to automate the web.

Instead of using Playwright (or other legacy frameworks) and having to parse through the DOM by hand to provide selectors in actions, Stagehand A/E/O lets you use natural language to build automations.

```text
// Playwright
await page.click('button[type="submit"]');

// Stagehand
stagehand.act("click the submit button")
```

This is helpful when writing scripts for the first time (development speed is much faster), but especially helpful for script maintenance. If a website changes and DOM selectors update, then Playwright scripts must be re-written to match the new page. Stagehand chooses selectors and actions at runtime, and are “self-healing”.

You can kinda see where we’re going with this. Jev fits extremely well into these primitives. We originally used an LLM (given context on what the page looked like and the goal) to decide what to do. With Jev we can use Choice to decide what selectors to interact with.

Lets use Act specifically to talk through the flow. Normally, we’d give the LLM a concise representation of the page using a hybrid a11y-tree. With Jev, we first [mark nodes](https://github.com/browserbase/stagehand/pull/2951) in the a11y tree as either interact-able (even rich-text editors) or not.

When stagehand.act is [called](https://github.com/browserbase/stagehand/pull/2953):

-   Jev classifies the instruction into an action (like click, fill, or scroll)
-   Stagehand parses arguments and builds candidate list for that action (which includes nearby page context
-   Jev answers “which candidate is best” and “does any candidate match” with an acceptance threshold of 0.7
-   If the candidate action is accepted then Stagehand handles the execution
-   If the action isn’t, then Stagehand falls back to an LLM

![Stagehand act flow: understand the instruction, select a candidate with Jev, then execute or fall back to an LLM.](https://cdn.sanity.io/images/yd6zslid/production/38761e1175bbddd7749be7dc678ec6298043c155-1702x276.jpg?auto=format&h=259&w=1600&fit=min&q=75)

_Act Flow_

In early testing, Act median latency drops from 1.97 seconds to 0.46 seconds which is about 4.3× faster (or 77% less time). [See the full PR stack.](https://github.com/browserbase/stagehand/pull/2993/changes)

With computer use, Jev is a piece of the pie but not a standalone solution. We’re now able to build more deterministic software tools that agents can use.

![When Jev is a good fit versus a poor fit, comparing structured decisions, latency, confidence, generation, and reasoning requirements.](https://cdn.sanity.io/images/yd6zslid/production/587092c017207c64eef93a1cb1bf509d127ce504-1854x802.png?fm=webp&h=692&w=1600&auto=format&fit=min&q=75)

_When to use Jev_

## Diffusing AI in the real world

Will Jev build production grade computer use agents? No. Is it a useful piece of the puzzle? I think it will be.

It feels like there's an abundance of ideas that didn't make sense before Jev. I’ve seen people build instant [search](https://x.com/dabit3/status/2100756930054504776), [smart copy paste](https://x.com/marcus_lowe/status/2101476399488160013), and more simple but extremely helpful tools.

AI shouldn’t be confined to some version of a chat interface, sync or async. With models like Jev, we can build software that incorporates prediction models without a chat input box. Even though classifiers have been available for so long they’ve never felt more useful. Maybe all we needed to build was inspiration.

\-> Kyle

## Start building with Browserbase

Run headless browsers for your agents and automations at scale. Get started free in minutes.

[Sign up for free](https://www.browserbase.com/sign-up)

## Keep reading

![](https://cdn.sanity.io/images/yd6zslid/production/2445455ec9974b1dfec01a87e446d1c99a3964cb-856x579.png?fm=webp&h=1082&w=1600&auto=format&fit=min&q=75)

[

### What is Web Bot Auth?

](/blog/what-is-web-bot-auth)

Authors

Harsehaj Dhami & Peyton Casper

Published on

September 30, 2026

Topic

Engineering

![](https://cdn.sanity.io/images/yd6zslid/production/4d82ed06235391a3af3562904943036c52162569-2568x1737.png?fm=webp&h=1082&w=1600&auto=format&fit=min&q=75)

[

### Browserbase and Okta: Cross App Access Ecosystem Brings Identity-Governed AI to browser agents

](/blog/browser-agent-identity-okta)

Published on

September 22, 2026

Topic

Engineering

![](https://cdn.sanity.io/images/yd6zslid/production/cd3a1c87bc3f777a2a15455ef582259e571d2432-856x579.png?fm=webp&h=1082&w=1600&auto=format&fit=min&q=75)

[

### Evolving computer use with code

](/blog/evolving-computer-use-with-code)

Authors

Kyle Jeong & Miguel Gonzalez

Published on

September 09, 2026

Topic

Engineering

![](https://cdn.sanity.io/images/yd6zslid/production/1929e225979d78d9c3007acb4b126b4d4accbf2a-856x579.png?fm=webp&h=1082&w=1600&auto=format&fit=min&q=75)

[

### Introducing Stagehand v4: The SDK for browser agents.

](/blog/stagehand-v4)

Authors

Miguel Gonzalez, Sean McGuire , Sam Finton & Harsehaj Dhami

Published on

August 10, 2026

Topic

Stagehand

[View all blog posts](/blog)