# Use cases

This document aims to describe what the goals of the users will be and how the
usage of the app should look like.

## Prerequisites and notes

To hamper spam and automation there will be a multi step process involving
E-Mail. This means every user needs to have access to an E-Mail account.
Additionally the users need to be identified, which will happen by some hashing
strategy involving the E-Mail and IP address of the user. By hashing the
information no personal data is stored.

| Keyword  | description    |
|-------------- | -------------- |
| ticket | An entry-ticket to the event which is bound to a specific seat. One per person is needed to be allowed to enter the event. Is reserved through the app but has to be collected in person    |
| access token | A short character sequence to identify users |
| order | Orders contain multiple tickets |

## Order tickets

1. The user has to request an access token via the web interface
2. An E-Mail is sent with the token
3. With this token the user can then select tickets to order via the web
   interface
4. The order will be processed and the user will get a code which identifies the
   order (via web interface and email)

### Restrictions

- Only one order per access code is allowed
- The amount of tickets that can be ordered is limited per access token
- Multiple access tokens can be provided together to be able to place bigger
  orders

## View order details

Details about an order, like which tickets are contained, can be accessed via
the web interface. It can either be reached by providing the order code on a
search page or by directly using the link in the order confirmation.

## Cancel an order

Orders can be canceled on the web interface. This will free up the access
token(s) which was/were linked to that order again.

